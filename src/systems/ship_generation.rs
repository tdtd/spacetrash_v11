use bevy::prelude::*;

use crate::components::*;
use crate::procgen::graph_enhanced_wfc::{
    graph_enhanced_collapse, GraphWfcBalanceProfile, GraphWfcConfig,
};
use crate::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode, RoomType, SystemFlags,
};
use crate::procgen::tile::{TileType, WallDirection};
use crate::resources::ship_template::ShipTemplate;
use crate::resources::{CurrentRoomGraph, CurrentSystemGraphs};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineDirection {
    EastWest,
    NorthSouth,
}

/// Deterministic pseudo-random generator used for room-type assignment and
/// interior decoration. Not cryptographically secure.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.max(1),
        }
    }

    fn next_f32(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 40) as f32 / ((1u64 << 24) as f32)
    }
}

/// Returns the transform for a floor plane so it lies flat on the XZ plane.
///
/// `Plane3d::default()` already lies flat, so no rotation is required.
fn floor_transform(pos: Vec3) -> Transform {
    Transform::from_translation(pos)
}

/// Returns the transform for a ceiling plane at the given height.
///
/// Rotate the floor plane 180° around X to face downward.
fn ceiling_transform(pos: Vec3, ceiling_height: f32) -> Transform {
    Transform::from_translation(Vec3::new(pos.x, ceiling_height, pos.z))
        .with_rotation(Quat::from_rotation_x(std::f32::consts::PI))
}

/// Returns the cuboid size used for a ceiling conduit line.
fn conduit_line_size(direction: LineDirection) -> Vec3 {
    match direction {
        LineDirection::EastWest => Vec3::new(1.0, 0.03, 0.03),
        LineDirection::NorthSouth => Vec3::new(0.03, 0.03, 1.0),
    }
}

/// Returns the cuboid size used for a floor pipe line.
fn pipe_line_size(direction: LineDirection) -> Vec3 {
    match direction {
        LineDirection::EastWest => Vec3::new(1.0, 0.02, 0.04),
        LineDirection::NorthSouth => Vec3::new(0.04, 0.02, 1.0),
    }
}

/// Spawns a ship entity and its rooms/walls from a `ShipTemplate`.
///
/// This is the first playable integration of the graph + GE-WFC generator.
/// The old grid-only generator has been replaced by:
/// 1. Build a planar `RoomGraph` from the template’s grid parameters.
/// 2. Run `graph_enhanced_collapse` to create a tile grid.
/// 3. Spawn one floor plane per floor/door tile and one directional wall per wall tile.
///
/// Walls use `WallSegmentCollider` and thin outward-facing visual slabs.
/// Door tiles now contain a floor plane and two side walls that form a
/// passage between adjacent rooms.
/// Conduit and pipe tiles now additionally spawn visible ceil/floor line meshes.
///
/// Floor, wall, and ceiling surfaces use a subtle checkerboard pattern to
/// separate them from one another and from gameplay-relevant geometry.
pub fn spawn_ship_from_template(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    template: &ShipTemplate,
    world_transform: Transform,
) -> Entity {
    let ship = commands
        .spawn((
            Ship,
            Transform::from_translation(world_transform.translation),
            Visibility::default(),
            GameplayEntity,
        ))
        .id();

    // Build a deterministic planar scaffold from the template.
    let mut graph = RoomGraph::new();

    let cols = template.grid_cols.max(1);
    let rows = template.grid_rows.max(1);
    let corridor_w = template.corridor_width;

    // Local RNG for deterministic room-type assignment, room-size variation,
    // and interior decoration.
    let mut rng = SimpleRng::new(template.seed);

    // Generate different room sizes per column/row. This is the first
    // step away from the old fixed-size “samey” ship. Rooms still form
    // a rectangular grid, but each column can have a different width and
    // each row can have a different height.
    let min_room_tiles = template.min_room_size.max(2.0).round() as usize;
    let max_room_tiles = template
        .max_room_size
        .max(template.min_room_size)
        .round() as usize;
    let width_range = max_room_tiles.saturating_sub(min_room_tiles) + 1;
    let height_range = max_room_tiles.saturating_sub(min_room_tiles) + 1;

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
            x += corridor_w;
        }
    }
    let total_w = x;

    let mut row_origins = Vec::with_capacity(rows);
    let mut y = 0.0f32;
    for row in 0..rows {
        row_origins.push(y);
        y += row_heights[row] as f32;
        if row < rows - 1 {
            y += corridor_w;
        }
    }
    let total_d = y;

    // Decide which room indices get the special room types.
    let total_rooms = cols * rows;
    let mut indices: Vec<usize> = (0..total_rooms).collect();
    // Fisher-Yates shuffle.
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

    // Grid coordinates: (0,0) is bottom-left of the ship bounding box.
    // World origin will be centered at the ship’s transform.
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

    // Add door edges between adjacent rooms.
    for row in 0..rows {
        for col in 0..cols {
            let id = (row * cols + col) as u32;

            // East neighbor
            if col + 1 < cols {
                let neighbor = (row * cols + col + 1) as u32;
                graph.add_edge(RoomEdge {
                    kind: EdgeKind::Door,
                    from: id,
                    to: neighbor,
                    location: EdgePlacement::East,
                });
            }

            // South neighbor
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

    // World-space conversion values; define them before grid generation so we
    // can store them in `CurrentRoomGraph`.
    let origin_x = -total_w * 0.5;
    let origin_z = -total_d * 0.5;

    // Generate the base tile grid with GE-WFC.
    let config = GraphWfcConfig {
        width: total_w.round() as usize,
        height: total_d.round() as usize,
        seed: template.seed,
    };
    let balance = GraphWfcBalanceProfile::default();

    let mut grid = match graph_enhanced_collapse(&config, &graph, &balance) {
        Ok(g) => g,
        Err(e) => {
            error!("GE-WFC failed: {:?}", e);
            // Fallback: empty ship with no floor.
            return ship;
        }
    };

    // Store the generated graph for schematic layers.
    commands.insert_resource(CurrentRoomGraph {
        graph: graph.clone(),
        origin_x,
        origin_z,
        width: config.width,
        height: config.height,
    });

    // Store the room-level system graphs.
    commands.insert_resource(derive_system_graphs(&graph));

    // Randomly decorate room interiors with conduit/pipe markers.
    // This complements the WFC-chosen markers and ensures every relevant
    // room gets at least some system lines.
    for room_node in graph.nodes() {
        let shape = &room_node.shape;
        let min_x = shape.x.floor() as i64;
        let max_x = (shape.x + shape.width).ceil() as i64;
        let min_y = shape.y.floor() as i64;
        let max_y = (shape.y + shape.height).ceil() as i64;

        let has_electrical = room_node.system_flags.contains(SystemFlags::ELECTRICAL);
        let has_fluidics = room_node.system_flags.contains(SystemFlags::FLUIDICS);

        if !has_electrical && !has_fluidics {
            continue;
        }

        for y in (min_y + 1)..(max_y - 1) {
            for x in (min_x + 1)..(max_x - 1) {
                if x < 0 || y < 0 || x >= config.width as i64 || y >= config.height as i64 {
                    continue;
                }

                let cell = &mut grid[y as usize][x as usize];

                if *cell != TileType::Floor {
                    continue;
                }

                if has_electrical && rng.next_f32() < 0.3 {
                    *cell = TileType::Conduit;
                } else if has_fluidics && rng.next_f32() < 0.3 {
                    *cell = TileType::Pipe;
                }
            }
        }
    }

    // Materials.
    let floor_material_a = materials.add(StandardMaterial {
        base_color: Color::rgb(0.42, 0.42, 0.42),
        ..default()
    });
    let floor_material_b = materials.add(StandardMaterial {
        base_color: Color::rgb(0.32, 0.32, 0.32),
        ..default()
    });
    let wall_material_a = materials.add(StandardMaterial {
        base_color: Color::rgb(0.24, 0.24, 0.24),
        ..default()
    });
    let wall_material_b = materials.add(StandardMaterial {
        base_color: Color::rgb(0.16, 0.16, 0.16),
        ..default()
    });
    let ceiling_material_a = materials.add(StandardMaterial {
        base_color: Color::rgb(0.52, 0.52, 0.52),
        ..default()
    });
    let ceiling_material_b = materials.add(StandardMaterial {
        base_color: Color::rgb(0.38, 0.38, 0.38),
        ..default()
    });
    let conduit_material = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 0.8, 0.0), // yellow-orange ceiling conduit
        ..default()
    });
    let pipe_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.6, 1.0), // blue floor pipe
        ..default()
    });

    // Spawn one entity per tile.
    for gy in 0..grid.len() {
        for gx in 0..grid[gy].len() {
            let tile = grid[gy][gx];
            let world_x = origin_x + gx as f32 + 0.5;
            let world_z = origin_z + gy as f32 + 0.5;
            let pos = Vec3::new(world_x, 0.0, world_z);

            let checker_even = (gx + gy) % 2 == 0;

            let floor_material = if checker_even {
                &floor_material_a
            } else {
                &floor_material_b
            };
            let wall_material = if checker_even {
                &wall_material_a
            } else {
                &wall_material_b
            };
            let ceiling_material = if checker_even {
                &ceiling_material_a
            } else {
                &ceiling_material_b
            };

            // Ceiling sits at the top of the wall slabs. Wall slabs are
            // centered at y=0 with height `wall_height`, so their top is
            // `wall_height / 2`.
            let ceiling_height = template.wall_height * 0.5;

            match tile {
                TileType::Floor => {
                    spawn_floor(
                        commands,
                        meshes,
                        floor_material,
                        pos,
                        tile,
                        GridCoordinates { x: gx as i32, y: gy as i32 },
                    );

                    spawn_ceiling_plane(
                        commands,
                        meshes,
                        ceiling_material,
                        pos,
                        ceiling_height,
                    );
                }
                TileType::Conduit => {
                    spawn_floor(
                        commands,
                        meshes,
                        floor_material,
                        pos,
                        tile,
                        GridCoordinates { x: gx as i32, y: gy as i32 },
                    );

                    spawn_ceiling_plane(
                        commands,
                        meshes,
                        ceiling_material,
                        pos,
                        ceiling_height,
                    );

                    let horizontal = (gx > 0 && grid[gy][gx - 1] == tile)
                        || (gx + 1 < grid[gy].len() && grid[gy][gx + 1] == tile);
                    let direction = if horizontal {
                        LineDirection::EastWest
                    } else {
                        LineDirection::NorthSouth
                    };

                    spawn_ceiling_conduit_line(
                        commands,
                        meshes,
                        &conduit_material,
                        pos,
                        ceiling_height,
                        direction,
                    );
                }
                TileType::Pipe => {
                    spawn_floor(
                        commands,
                        meshes,
                        floor_material,
                        pos,
                        tile,
                        GridCoordinates { x: gx as i32, y: gy as i32 },
                    );

                    spawn_ceiling_plane(
                        commands,
                        meshes,
                        ceiling_material,
                        pos,
                        ceiling_height,
                    );

                    let horizontal = (gx > 0 && grid[gy][gx - 1] == tile)
                        || (gx + 1 < grid[gy].len() && grid[gy][gx + 1] == tile);
                    let direction = if horizontal {
                        LineDirection::EastWest
                    } else {
                        LineDirection::NorthSouth
                    };

                    spawn_floor_pipe_line(commands, meshes, &pipe_material, pos, direction);
                }
                TileType::Door(direction) => {
                    spawn_floor(
                        commands,
                        meshes,
                        floor_material,
                        pos,
                        tile,
                        GridCoordinates { x: gx as i32, y: gy as i32 },
                    );

                    spawn_ceiling_plane(
                        commands,
                        meshes,
                        ceiling_material,
                        pos,
                        ceiling_height,
                    );

                    spawn_door_side_walls(
                        commands,
                        meshes,
                        wall_material,
                        template.wall_height,
                        template.wall_thickness,
                        direction,
                        pos,
                    );
                }
                TileType::Wall(direction) => {
                    spawn_floor_surface_only(commands, meshes, floor_material, pos);

                    let wall_collider = WallSegmentCollider::for_direction(direction);
                    let visual_assets = wall_visual_assets(
                        direction,
                        template.wall_height,
                        template.wall_thickness,
                        &mut *meshes,
                    );

                    let wall_entity = commands
                        .spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(pos),
                                ..default()
                            },
                            wall_collider,
                            GameplayEntity,
                            TileNode { tile_type: tile },
                            GridCoordinates { x: gx as i32, y: gy as i32 },
                        ))
                        .id();

                    commands.entity(wall_entity).with_children(|parent| {
                        for (mesh, visual_transform) in visual_assets {
                            parent.spawn(PbrBundle {
                                mesh,
                                material: wall_material.clone(),
                                transform: visual_transform,
                                ..default()
                            });
                        }
                    });
                }
                TileType::Void => {
                    // Deadzone/outside ship. Intentionally left empty.
                }
            }
        }
    }

    // Spawn room entities for schematic maps, graph connectivity, and future gameplay.
    for room_node in graph.nodes() {
        let rect = &room_node.shape;
        let center_grid_x = rect.x + rect.width * 0.5;
        let center_grid_y = rect.y + rect.height * 0.5;

        let world_x = origin_x + center_grid_x;
        let world_z = origin_z + center_grid_y;

        commands.spawn((
            Room {
                width: rect.width,
                height: rect.height,
                ship_entity: Some(ship),
            },
            Transform::from_xyz(world_x, 0.0, world_z),
            GameplayEntity,
        ));
    }

    // Docking ports remain unchanged.
    for port_def in &template.docking_ports {
        let local_pos = Vec3::new(port_def.local_position[0], 0.0, port_def.local_position[1]);
        commands.spawn((
            DockingPort::new(ship, local_pos),
            Transform::from_translation(local_pos),
            GameplayEntity,
        ));
    }

    ship
}

/// Derives room-level system graphs from the scaffold.
///
/// Later this will also incorporate WFC tile-level semantic connections.
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

/// Creates one or two thin slabs for a directional wall.
///
/// Each slab is placed flush with the tile’s outer face, so from inside the
/// room the wall appears as a thin hull rather than a full block. Collision
/// still comes from `WallSegmentCollider` on the inward tile edge.
fn wall_visual_assets(
    direction: WallDirection,
    wall_height: f32,
    wall_thickness: f32,
    meshes: &mut Assets<Mesh>,
) -> Vec<(Handle<Mesh>, Transform)> {
    let thickness = wall_thickness.max(0.05);
    let slab_h = wall_height;
    let half_thickness = thickness * 0.5;
    let mut assets = Vec::new();

    let mut add_slab =
        |size: Vec3, translation: Vec3, assets: &mut Vec<(Handle<Mesh>, Transform)>| {
            assets.push((
                meshes.add(Cuboid::new(size.x, size.y, size.z)),
                Transform::from_translation(translation),
            ));
        };

    match direction {
        WallDirection::North => add_slab(
            Vec3::new(1.0, slab_h, thickness),
            Vec3::new(0.0, 0.0, 0.5 - half_thickness),
            &mut assets,
        ),
        WallDirection::South => add_slab(
            Vec3::new(1.0, slab_h, thickness),
            Vec3::new(0.0, 0.0, -0.5 + half_thickness),
            &mut assets,
        ),
        WallDirection::East => add_slab(
            Vec3::new(thickness, slab_h, 1.0),
            Vec3::new(0.5 - half_thickness, 0.0, 0.0),
            &mut assets,
        ),
        WallDirection::West => add_slab(
            Vec3::new(thickness, slab_h, 1.0),
            Vec3::new(-0.5 + half_thickness, 0.0, 0.0),
            &mut assets,
        ),
        WallDirection::NorthEast => {
            add_slab(
                Vec3::new(thickness, slab_h, 1.0),
                Vec3::new(0.5 - half_thickness, 0.0, 0.0),
                &mut assets,
            );
            add_slab(
                Vec3::new(1.0, slab_h, thickness),
                Vec3::new(0.0, 0.0, 0.5 - half_thickness),
                &mut assets,
            );
        }
        WallDirection::NorthWest => {
            add_slab(
                Vec3::new(thickness, slab_h, 1.0),
                Vec3::new(-0.5 + half_thickness, 0.0, 0.0),
                &mut assets,
            );
            add_slab(
                Vec3::new(1.0, slab_h, thickness),
                Vec3::new(0.0, 0.0, 0.5 - half_thickness),
                &mut assets,
            );
        }
        WallDirection::SouthEast => {
            add_slab(
                Vec3::new(thickness, slab_h, 1.0),
                Vec3::new(0.5 - half_thickness, 0.0, 0.0),
                &mut assets,
            );
            add_slab(
                Vec3::new(1.0, slab_h, thickness),
                Vec3::new(0.0, 0.0, -0.5 + half_thickness),
                &mut assets,
            );
        }
        WallDirection::SouthWest => {
            add_slab(
                Vec3::new(thickness, slab_h, 1.0),
                Vec3::new(-0.5 + half_thickness, 0.0, 0.0),
                &mut assets,
            );
            add_slab(
                Vec3::new(1.0, slab_h, thickness),
                Vec3::new(0.0, 0.0, -0.5 + half_thickness),
                &mut assets,
            );
        }
    }

    assets
}

/// Spawns a floor plane with the given tile type and no children.
///
/// Floor material is chosen externally to provide a checkerboard variation.
fn spawn_floor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    floor_material: &Handle<StandardMaterial>,
    pos: Vec3,
    tile: TileType,
    grid_coords: GridCoordinates,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
            material: floor_material.clone(),
            transform: floor_transform(pos),
            ..default()
        },
        GameplayEntity,
        TileNode { tile_type: tile },
        grid_coords,
    ));
}

/// Spawns a floor plane without a `TileNode` component.
///
/// This is used for floor surfaces underneath wall tiles so the wall entity
/// remains the only tile data source.
fn spawn_floor_surface_only(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    floor_material: &Handle<StandardMaterial>,
    pos: Vec3,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
            material: floor_material.clone(),
            transform: floor_transform(pos),
            ..default()
        },
        GameplayEntity,
    ));
}

/// Spawns a ceiling plane for open room tiles.
///
/// Ceilings are only visible from inside the ship; this crate does not use
/// a top-down/bird's-eye camera.
fn spawn_ceiling_plane(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    ceiling_material: &Handle<StandardMaterial>,
    pos: Vec3,
    ceiling_height: f32,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
            material: ceiling_material.clone(),
            transform: ceiling_transform(pos, ceiling_height),
            ..default()
        },
        GameplayEntity,
    ));
}

/// Spawns two side-wall entities for a door tile.
///
/// For horizontal door tiles (East/West), the side walls are placed on the
/// north and south edges. For vertical door tiles (North/South), they are
/// placed on the east and west edges. This forms a passage between adjacent
/// rooms while blocking the surrounding deadzone.
fn spawn_door_side_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    wall_material: &Handle<StandardMaterial>,
    wall_height: f32,
    wall_thickness: f32,
    direction: WallDirection,
    center: Vec3,
) {
    let thickness = wall_thickness.max(0.05);
    let slab_h = wall_height;
    let half_thickness = thickness * 0.5;

    let placements = match direction {
        WallDirection::East | WallDirection::West => vec![
            (
                Vec3::new(0.0, 0.0, 0.5 - half_thickness),
                Vec3::new(1.0, slab_h, thickness),
            ),
            (
                Vec3::new(0.0, 0.0, -0.5 + half_thickness),
                Vec3::new(1.0, slab_h, thickness),
            ),
        ],
        WallDirection::North | WallDirection::South => vec![
            (
                Vec3::new(0.5 - half_thickness, 0.0, 0.0),
                Vec3::new(thickness, slab_h, 1.0),
            ),
            (
                Vec3::new(-0.5 + half_thickness, 0.0, 0.0),
                Vec3::new(thickness, slab_h, 1.0),
            ),
        ],
        _ => vec![],
    };

    for (offset, size) in placements {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
                material: wall_material.clone(),
                transform: Transform::from_translation(center + offset),
                ..default()
            },
            AabbCollider {
                half_extents: Vec3::new(size.x * 0.5, size.y * 0.5, size.z * 0.5),
            },
            GameplayEntity,
        ));
    }
}

/// Spawns a thin ceiling conduit line over a conduit tile.
///
/// Represents a visible electrical wire/conduit running at ceiling height.
fn spawn_ceiling_conduit_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    conduit_material: &Handle<StandardMaterial>,
    center: Vec3,
    ceiling_height: f32,
    direction: LineDirection,
) {
    let y = (ceiling_height - 0.05).max(0.05);
    let size = conduit_line_size(direction);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
            material: conduit_material.clone(),
            transform: Transform::from_xyz(center.x, y, center.z),
            ..default()
        },
        GameplayEntity,
    ));
}

/// Spawns a thin floor pipe line over a pipe tile.
///
/// Represents a visible fluid pipe running along the floor.
fn spawn_floor_pipe_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    pipe_material: &Handle<StandardMaterial>,
    center: Vec3,
    direction: LineDirection,
) {
    let y = 0.03;
    let size = pipe_line_size(direction);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
            material: pipe_material.clone(),
            transform: Transform::from_xyz(center.x, y, center.z),
            ..default()
        },
        GameplayEntity,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_transform_leaves_plane_flat() {
        let expected = Quat::IDENTITY;
        let actual = floor_transform(Vec3::ZERO).rotation;

        assert_eq!(actual, expected);
    }

    #[test]
    fn ceiling_transform_rotates_plane_downward() {
        let height = 2.0;
        let transform = ceiling_transform(Vec3::new(1.0, 0.0, 2.0), height);

        assert_eq!(
            transform.rotation,
            Quat::from_rotation_x(std::f32::consts::PI)
        );
        assert_eq!(
            transform.translation,
            Vec3::new(1.0, height, 2.0)
        );
    }

    #[test]
    fn conduit_line_size_depends_on_direction() {
        assert_eq!(
            conduit_line_size(LineDirection::EastWest),
            Vec3::new(1.0, 0.03, 0.03)
        );
        assert_eq!(
            conduit_line_size(LineDirection::NorthSouth),
            Vec3::new(0.03, 0.03, 1.0)
        );
    }

    #[test]
    fn pipe_line_size_depends_on_direction() {
        assert_eq!(
            pipe_line_size(LineDirection::EastWest),
            Vec3::new(1.0, 0.02, 0.04)
        );
        assert_eq!(
            pipe_line_size(LineDirection::NorthSouth),
            Vec3::new(0.04, 0.02, 1.0)
        );
    }
}
