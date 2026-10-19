use std::collections::{HashMap, HashSet};

use crate::procgen::scaffold::{RoomGraph, EdgeKind, SystemFlags};
use crate::procgen::tile::{TileType, WallDirection};
use crate::procgen::wfc::{
    run_wfc_with_settings, TileAdjacency, TileId, WfcBalanceProfile, WfcGrid, WfcSettings,
    WfcTileSet,
};

/// Global WFC configuration for the full ship grid.
#[derive(Debug, Clone)]
pub struct GraphWfcConfig {
    pub width: usize,
    pub height: usize,
    pub seed: u64,
}

/// Balance knobs specific to Graph Enhanced WFC.
#[derive(Debug, Clone, Default)]
pub struct GraphWfcBalanceProfile {
    pub tile_weight_multipliers: HashMap<TileId, f32>,
}

/// Errors produced by Graph Enhanced WFC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphWfcError {
    InvalidDimensions,
    Contradiction {
        x: usize,
        y: usize,
        message: String,
    },
}

const T_FLOOR: TileId = 0;
const T_CONDUIT: TileId = 1;
const T_PIPE: TileId = 2;
const T_WALL_N: TileId = 3;
const T_WALL_S: TileId = 4;
const T_WALL_E: TileId = 5;
const T_WALL_W: TileId = 6;
const T_WALL_NE: TileId = 7;
const T_WALL_NW: TileId = 8;
const T_WALL_SE: TileId = 9;
const T_WALL_SW: TileId = 10;
const T_DOOR_N: TileId = 11;
const T_DOOR_S: TileId = 12;
const T_DOOR_E: TileId = 13;
const T_DOOR_W: TileId = 14;
const T_VOID: TileId = 15;

fn all_tiles() -> HashSet<TileId> {
    (T_FLOOR..=T_VOID).collect()
}

fn hs(tiles: &[TileId]) -> HashSet<TileId> {
    tiles.iter().copied().collect()
}

fn build_semantic_tileset() -> (HashMap<TileId, TileType>, WfcTileSet) {
    let mut type_map = HashMap::new();
    type_map.insert(T_FLOOR, TileType::Floor);
    type_map.insert(T_CONDUIT, TileType::Conduit);
    type_map.insert(T_PIPE, TileType::Pipe);
    type_map.insert(T_WALL_N, TileType::Wall(WallDirection::North));
    type_map.insert(T_WALL_S, TileType::Wall(WallDirection::South));
    type_map.insert(T_WALL_E, TileType::Wall(WallDirection::East));
    type_map.insert(T_WALL_W, TileType::Wall(WallDirection::West));
    type_map.insert(T_WALL_NE, TileType::Wall(WallDirection::NorthEast));
    type_map.insert(T_WALL_NW, TileType::Wall(WallDirection::NorthWest));
    type_map.insert(T_WALL_SE, TileType::Wall(WallDirection::SouthEast));
    type_map.insert(T_WALL_SW, TileType::Wall(WallDirection::SouthWest));
    type_map.insert(T_DOOR_N, TileType::Door(WallDirection::North));
    type_map.insert(T_DOOR_S, TileType::Door(WallDirection::South));
    type_map.insert(T_DOOR_E, TileType::Door(WallDirection::East));
    type_map.insert(T_DOOR_W, TileType::Door(WallDirection::West));
    type_map.insert(T_VOID, TileType::Wall(WallDirection::North));

    let void_neighbors = hs(&[
        T_VOID, T_WALL_N, T_WALL_S, T_WALL_E, T_WALL_W,
        T_WALL_NE, T_WALL_NW, T_WALL_SE, T_WALL_SW,
    ]);

    let open_neighbors = hs(&[
        T_FLOOR,
        T_CONDUIT,
        T_PIPE,
        T_WALL_N,
        T_WALL_S,
        T_WALL_E,
        T_WALL_W,
        T_WALL_NE,
        T_WALL_NW,
        T_WALL_SE,
        T_WALL_SW,
        T_DOOR_N,
        T_DOOR_S,
        T_DOOR_E,
        T_DOOR_W,
    ]);

    let wall_neighbors = all_tiles();

    let mut adjacency = HashMap::new();

    adjacency.insert(
        T_VOID,
        TileAdjacency {
            north: void_neighbors.clone(),
            south: void_neighbors.clone(),
            east: void_neighbors.clone(),
            west: void_neighbors,
        },
    );

    for open in [T_FLOOR, T_CONDUIT, T_PIPE, T_DOOR_N, T_DOOR_S, T_DOOR_E, T_DOOR_W] {
        adjacency.insert(
            open,
            TileAdjacency {
                north: open_neighbors.clone(),
                south: open_neighbors.clone(),
                east: open_neighbors.clone(),
                west: open_neighbors.clone(),
            },
        );
    }

    for wall in [
        T_WALL_N, T_WALL_S, T_WALL_E, T_WALL_W,
        T_WALL_NE, T_WALL_NW, T_WALL_SE, T_WALL_SW,
    ] {
        adjacency.insert(
            wall,
            TileAdjacency {
                north: wall_neighbors.clone(),
                south: wall_neighbors.clone(),
                east: wall_neighbors.clone(),
                west: wall_neighbors.clone(),
            },
        );
    }

    let tile_set = WfcTileSet::new(all_tiles(), adjacency);
    (type_map, tile_set)
}

fn singleton(tile: TileId) -> HashSet<TileId> {
    HashSet::from([tile])
}

fn build_graph_domains(
    config: &GraphWfcConfig,
    graph: &RoomGraph,
) -> Vec<Vec<HashSet<TileId>>> {
    let width = config.width;
    let height = config.height;

    let mut domains = vec![vec![all_tiles(); width]; height];
    let mut covered = vec![vec![false; width]; height];

    // Room interiors and boundaries.
    for room in graph.nodes() {
        let shape = &room.shape;
        let min_x = shape.x.floor() as i64;
        let max_x = (shape.x + shape.width).ceil() as i64;
        let min_y = shape.y.floor() as i64;
        let max_y = (shape.y + shape.height).ceil() as i64;

        let has_electrical = room.system_flags.contains(SystemFlags::ELECTRICAL);
        let has_fluidics = room.system_flags.contains(SystemFlags::FLUIDICS);

        for y in min_y..max_y {
            for x in min_x..max_x {
                if x < 0 || y < 0 || x >= width as i64 || y >= height as i64 {
                    continue;
                }

                let x = x as usize;
                let y = y as usize;
                covered[y][x] = true;

                let is_left = x == min_x as usize;
                let is_right = x == max_x as usize - 1;
                let is_bottom = y == min_y as usize;
                let is_top = y == max_y as usize - 1;

                if is_top && is_left {
                    domains[y][x] = singleton(T_WALL_NW);
                } else if is_top && is_right {
                    domains[y][x] = singleton(T_WALL_NE);
                } else if is_bottom && is_left {
                    domains[y][x] = singleton(T_WALL_SW);
                } else if is_bottom && is_right {
                    domains[y][x] = singleton(T_WALL_SE);
                } else if is_left {
                    domains[y][x] = singleton(T_WALL_W);
                } else if is_right {
                    domains[y][x] = singleton(T_WALL_E);
                } else if is_bottom {
                    domains[y][x] = singleton(T_WALL_S);
                } else if is_top {
                    domains[y][x] = singleton(T_WALL_N);
                } else {
                    let mut interior_tiles = vec![T_FLOOR];

                    if has_electrical {
                        interior_tiles.push(T_CONDUIT);
                    }

                    if has_fluidics {
                        interior_tiles.push(T_PIPE);
                    }

                    domains[y][x] = hs(&interior_tiles);
                }
            }
        }
    }

    // Non-room cells default to void.
    for y in 0..height {
        for x in 0..width {
            if !covered[y][x] {
                domains[y][x] = singleton(T_VOID);
            }
        }
    }

    // Door/corridor cells override the deadzone and adjacent room wall cells.
    for edge in graph.edges() {
        if edge.kind != EdgeKind::Door
            && edge.kind != EdgeKind::Corridor
            && edge.kind != EdgeKind::Airlock
            && edge.kind != EdgeKind::Umbilical
        {
            continue;
        }

        let Some(from_room) = graph.get_node(edge.from) else {
            continue;
        };
        let Some(to_room) = graph.get_node(edge.to) else {
            continue;
        };

        let center_a_x = from_room.shape.x + from_room.shape.width * 0.5;
        let center_a_y = from_room.shape.y + from_room.shape.height * 0.5;
        let center_b_x = to_room.shape.x + to_room.shape.width * 0.5;
        let center_b_y = to_room.shape.y + to_room.shape.height * 0.5;

        let door_x = ((center_a_x + center_b_x) * 0.5).floor() as i64;
        let door_y = ((center_a_y + center_b_y) * 0.5).floor() as i64;

        if door_x < 0 || door_y < 0 || door_x >= width as i64 || door_y >= height as i64 {
            continue;
        }

        let horizontal = (center_a_y - center_b_y).abs() < (center_a_x - center_b_x).abs();
        let door_tile = if horizontal { T_DOOR_E } else { T_DOOR_N };

        let door_x = door_x as usize;
        let door_y = door_y as usize;
        domains[door_y][door_x] = singleton(door_tile);

        // Open the wall cells immediately on each side, matching the
        // corridor/deadzone model used by the previous floorplan builder.
        if horizontal {
            if door_x > 0 {
                domains[door_y][door_x - 1] = hs(&[T_FLOOR, T_CONDUIT, T_PIPE]);
            }
            if door_x + 1 < width {
                domains[door_y][door_x + 1] = hs(&[T_FLOOR, T_CONDUIT, T_PIPE]);
            }
        } else {
            if door_y > 0 {
                domains[door_y - 1][door_x] = hs(&[T_FLOOR, T_CONDUIT, T_PIPE]);
            }
            if door_y + 1 < height {
                domains[door_y + 1][door_x] = hs(&[T_FLOOR, T_CONDUIT, T_PIPE]);
            }
        }
    }

    domains
}

/// Runs graph-enhanced WFC over the full ship grid.
///
/// This version uses the room graph as a non-local skeleton: per-cell domains
/// preserve required wall/door structure, while interior domains are filtered
/// to only allow system tiles appropriate to each room's `SystemFlags`.
///
/// The actual collapse is seeded and weighted, so identical configurations
/// reproduce identical maps, while different seeds can create variety.
pub fn graph_enhanced_collapse(
    config: &GraphWfcConfig,
    graph: &RoomGraph,
    balance: &GraphWfcBalanceProfile,
) -> Result<Vec<Vec<TileType>>, GraphWfcError> {
    if config.width == 0 || config.height == 0 {
        return Err(GraphWfcError::InvalidDimensions);
    }

    let (type_map, tile_set) = build_semantic_tileset();
    let domains = build_graph_domains(config, graph);

    let mut wfc_grid = WfcGrid::new_with_domains(config.width, config.height, domains);

    let mut wfc_balance = WfcBalanceProfile::default();
    wfc_balance.tile_weight_multipliers = balance.tile_weight_multipliers.clone();

    let settings = WfcSettings { seed: config.seed };

    let wfc_result = run_wfc_with_settings(&mut wfc_grid, &tile_set, &settings, &wfc_balance)
        .map_err(|message| GraphWfcError::Contradiction {
            x: 0,
            y: 0,
            message,
        })?;

    let mut output = Vec::with_capacity(config.height);
    for y in 0..config.height {
        let mut row = Vec::with_capacity(config.width);
        for x in 0..config.width {
            let tile_id = wfc_result.tiles[y][x];
            let tile_type = type_map
                .get(&tile_id)
                .copied()
                .unwrap_or(TileType::Wall(WallDirection::North));
            row.push(tile_type);
        }
        output.push(row);
    }

    Ok(output)
}
