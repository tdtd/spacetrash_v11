use bevy::prelude::*;

use crate::components::*;
use crate::procgen::graph_enhanced_wfc::{
    graph_enhanced_collapse, GraphWfcBalanceProfile, GraphWfcConfig,
};
use crate::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode, RoomType, SystemFlags,
};
use crate::procgen::tile::TileType;
use crate::resources::{CurrentRoomGraph, CurrentSystemGraphs};
use crate::systems::set_pieces::{set_piece_kind_for_room, set_piece_half_extents};

use super::config::{ArenaActorSpawn, ArenaConfig, ArenaShipConfig};

/// Generated ship data reused by both tests and world spawning.
pub struct ShipData {
    pub graph: RoomGraph,
    pub grid: Vec<Vec<TileType>>,
    pub origin_x: f32,
    pub origin_z: f32,
    pub width: usize,
    pub height: usize,
}

/// Small deterministic RNG, intentionally separate from gameplay code.
struct ArenaRng(u64);

impl ArenaRng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 40) as f32 / ((1u64 << 24) as f32)
    }
}

/// Builds a deterministic room graph matching the current ship generator.
fn build_room_graph(ship: &ArenaShipConfig) -> RoomGraph {
    let mut graph = RoomGraph::new();
    let cols = ship.grid_cols.max(1);
    let rows = ship.grid_rows.max(1);

    let min_room_tiles = ship.min_room_size.max(2.0).round() as usize;
    let max_room_tiles = ship.max_room_size.max(ship.min_room_size).round() as usize;
    let width_range = max_room_tiles.saturating_sub(min_room_tiles) + 1;
    let height_range = max_room_tiles.saturating_sub(min_room_tiles) + 1;

    let mut rng = ArenaRng::new(ship.seed);

    let col_widths: Vec<usize> = (0..cols)
        .map(|_| min_room_tiles + ((rng.next_f32() * width_range as f32) as usize))
        .collect();

    let row_heights: Vec<usize> = (0..rows)
        .map(|_| min_room_tiles + ((rng.next_f32() * height_range as f32) as usize))
        .collect();

    let mut col_origins = Vec::with_capacity(cols);
    let mut x = 0.0f32;
    for col in 0..cols {
        col_origins.push(x);
        x += col_widths[col] as f32;
        if col < cols - 1 {
            x += ship.corridor_width;
        }
    }

    let mut row_origins = Vec::with_capacity(rows);
    let mut y = 0.0f32;
    for row in 0..rows {
        row_origins.push(y);
        y += row_heights[row] as f32;
        if row < rows - 1 {
            y += ship.corridor_width;
        }
    }

    let total_rooms = cols * rows;
    let mut indices: Vec<usize> = (0..total_rooms).collect();
    for i in (1..total_rooms).rev() {
        let j = (rng.next_f32() * (i + 1) as f32) as usize;
        indices.swap(i, j);
    }

    let special_room_types = [
        RoomType::Reactor,
        RoomType::Bridge,
        RoomType::LifeSupport,
        RoomType::Airlock,
    ];
    let mut room_type_by_idx = vec![RoomType::Storage; total_rooms];
    for (k, &special) in special_room_types.iter().take(total_rooms).enumerate() {
        room_type_by_idx[indices[k]] = special;
    }

    for row in 0..rows {
        for col in 0..cols {
            let id = (row * cols + col) as u32;
            let idx = row * cols + col;
            let room_type = room_type_by_idx[idx];

            let rect = Rect::new(
                col_origins[col],
                row_origins[row],
                col_widths[col] as f32,
                row_heights[row] as f32,
            );

            graph.add_node(RoomNode::new(id, room_type, rect));
        }
    }

    for row in 0..rows {
        for col in 0..cols {
            let id = (row * cols + col) as u32;

            if col + 1 < cols {
                let neighbor = (row * cols + col + 1) as u32;
                graph.add_edge(RoomEdge {
                    kind: EdgeKind::Door,
                    from: id,
                    to: neighbor,
                    location: EdgePlacement::East,
                });
            }

            if row + 1 < rows {
                let neighbor = ((row + 1) * cols + col) as u32;
                graph.add_edge(RoomEdge {
                    kind: EdgeKind::Door,
                    from: id,
                    to: neighbor,
                    location: EdgePlacement::South,
                });
            }
        }
    }

    graph
}

/// Generates the full graph + collapsed tile grid for an arena ship.
pub fn generate_ship_data(ship: &ArenaShipConfig) -> Result<ShipData, String> {
    let graph = build_room_graph(ship);

    let mut total_w = 0.0f32;
    let mut total_d = 0.0f32;

    for node in graph.nodes() {
        total_w = total_w.max(node.shape.x + node.shape.width);
        total_d = total_d.max(node.shape.y + node.shape.height);
    }

    let wfc_config = GraphWfcConfig {
        width: total_w.round() as usize,
        height: total_d.round() as usize,
        seed: ship.seed,
    };

    let balance = GraphWfcBalanceProfile::default();
    let grid = graph_enhanced_collapse(&wfc_config, &graph, &balance)
        .map_err(|e| format!("GE-WFC failed: {e:?}"))?;

    Ok(ShipData {
        origin_x: -total_w * 0.5,
        origin_z: -total_d * 0.5,
        width: grid[0].len(),
        height: grid.len(),
        graph,
        grid,
    })
}

/// Spawns an actor into the world using the same component set as the
/// existing deterministic test harness.
pub fn spawn_arena_actor(world: &mut World, actor: &ArenaActorSpawn) {
    world.spawn((
        Transform::from_translation(actor.translation),
        Velocity(actor.velocity),
        Rotation { yaw: actor.yaw },
        AngularVelocity {
            yaw: actor.angular_velocity,
        },
        CircleCollider { radius: actor.radius },
        GameplayEntity,
    ));
}

/// Spawns all gameplay-relevant ship entities without render assets.
fn spawn_ship_in_world(world: &mut World, ship: &ArenaShipConfig, data: &ShipData) {
    let ship_entity = world
        .spawn((
            Ship,
            Transform::default(),
            Visibility::default(),
            GameplayEntity,
        ))
        .id();

    world.insert_resource(CurrentRoomGraph {
        graph: data.graph.clone(),
        origin_x: data.origin_x,
        origin_z: data.origin_z,
        width: data.width,
        height: data.height,
    });

    world.insert_resource(derive_system_graphs(&data.graph));

    for room in data.graph.nodes() {
        let world_x = data.origin_x + room.shape.x + room.shape.width * 0.5;
        let world_z = data.origin_z + room.shape.y + room.shape.height * 0.5;

        world.spawn((
            Room {
                width: room.shape.width,
                height: room.shape.height,
                ship_entity: Some(ship_entity),
            },
            Transform::from_xyz(world_x, 0.0, world_z),
            GameplayEntity,
        ));

        if let Some(kind) = set_piece_kind_for_room(room.room_type) {
            let half_extents = set_piece_half_extents(kind);

            if kind == SetPieceKind::ReactorCore {
                world.spawn((
                    PhysicalSetPiece,
                    kind,
                    AabbCollider { half_extents },
                    Transform::from_xyz(world_x, 0.0, world_z),
                    GameplayEntity,
                    ThermalEmitter {
                        intensity: 3.0,
                        range: 4.0,
                    },
                    HeatAccumulator::default(),
                ));
            } else {
                world.spawn((
                    PhysicalSetPiece,
                    kind,
                    AabbCollider { half_extents },
                    Transform::from_xyz(world_x, 0.0, world_z),
                    GameplayEntity,
                ));
            }
        }
    }

    for gy in 0..data.grid.len() {
        for gx in 0..data.grid[gy].len() {
            let tile = data.grid[gy][gx];
            let world_x = data.origin_x + gx as f32 + 0.5;
            let world_z = data.origin_z + gy as f32 + 0.5;
            let pos = Vec3::new(world_x, 0.0, world_z);

            match tile {
                TileType::Floor | TileType::Conduit | TileType::Pipe | TileType::Door(_) => {
                    world.spawn((
                        Transform::from_translation(pos),
                        TileNode { tile_type: tile },
                        GridCoordinates {
                            x: gx as i32,
                            y: gy as i32,
                        },
                        GameplayEntity,
                    ));
                }
                TileType::Wall(direction) => {
                    world.spawn((
                        Transform::from_translation(pos),
                        WallSegmentCollider::for_direction(direction),
                        TileNode { tile_type: tile },
                        GridCoordinates {
                            x: gx as i32,
                            y: gy as i32,
                        },
                        GameplayEntity,
                    ));
                }
                TileType::Void => {
                    // Deadzone/outside ship. Intentionally left empty.
                }
            }
        }
    }

    let _ = ship.wall_thickness;
}

fn derive_system_graphs(graph: &RoomGraph) -> CurrentSystemGraphs {
    let mut graphs = CurrentSystemGraphs::default();

    for edge in graph.edges() {
        let Some(a) = graph.get_node(edge.from) else {
            continue;
        };
        let Some(b) = graph.get_node(edge.to) else {
            continue;
        };

        if a.system_flags.contains(SystemFlags::ELECTRICAL)
            && b.system_flags.contains(SystemFlags::ELECTRICAL)
        {
            graphs.electrical.entry(a.id).or_default().push(b.id);
            graphs.electrical.entry(b.id).or_default().push(a.id);
        }

        if a.system_flags.contains(SystemFlags::FLUIDICS)
            && b.system_flags.contains(SystemFlags::FLUIDICS)
        {
            graphs.fluidics.entry(a.id).or_default().push(b.id);
            graphs.fluidics.entry(b.id).or_default().push(a.id);
        }

        if a.system_flags.contains(SystemFlags::ATMOSPHERICS)
            && b.system_flags.contains(SystemFlags::ATMOSPHERICS)
        {
            graphs.atmospherics.entry(a.id).or_default().push(b.id);
            graphs.atmospherics.entry(b.id).or_default().push(a.id);
        }

        if a.system_flags.contains(SystemFlags::THERMAL)
            && b.system_flags.contains(SystemFlags::THERMAL)
        {
            graphs.thermal.entry(a.id).or_default().push(b.id);
            graphs.thermal.entry(b.id).or_default().push(a.id);
        }

        if a.system_flags.contains(SystemFlags::COMPUTER)
            && b.system_flags.contains(SystemFlags::COMPUTER)
        {
            graphs.computer.entry(a.id).or_default().push(b.id);
            graphs.computer.entry(b.id).or_default().push(a.id);
        }
    }

    graphs
}

/// Clears all `GameplayEntity` entities currently in the world.
fn despawn_gameplay_world(world: &mut World) {
    let entities: Vec<Entity> = world
        .query::<(Entity, &GameplayEntity)>()
        .iter(world)
        .map(|(entity, _)| entity)
        .collect();

    for entity in entities {
        world.despawn(entity);
    }
}

/// Builds or resets the arena world from `config`.
///
/// This is intentionally render-free and therefore safe for headless
/// deterministic runners.
pub fn build_arena_world(world: &mut World, config: &ArenaConfig) {
    despawn_gameplay_world(world);

    let _ = world.remove_resource::<CurrentRoomGraph>();
    let _ = world.remove_resource::<CurrentSystemGraphs>();

    if let Some(ship) = &config.ship {
        match generate_ship_data(ship) {
            Ok(data) => spawn_ship_in_world(world, ship, &data),
            Err(err) => {
                bevy::log::warn!("Arena ship generation failed: {err}");
            }
        }
    }

    for actor in &config.actors {
        spawn_arena_actor(world, actor);
    }
}

/// Alias for `build_arena_world`, matching the roadmap naming.
pub fn reset_arena_world(world: &mut World, config: &ArenaConfig) {
    build_arena_world(world, config);
}
