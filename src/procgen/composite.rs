use std::collections::HashMap;

use crate::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode,
};
use crate::procgen::tile::TileType;

/// One generated ship/station instance that will be merged into a composite level.
///
/// `graph` and `grid` are in local tile coordinates.
/// `offset_x` and `offset_y` specify where the local tile `(0, 0)` will land
/// in the global composite grid.
#[derive(Debug, Clone)]
pub struct InstancePlacement {
    pub graph: RoomGraph,
    pub grid: Vec<Vec<TileType>>,
    pub offset_x: i32,
    pub offset_y: i32,
}

/// A connection between two instance-local room IDs that should become a
/// `Umbilical` edge in the merged graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UmbilicalConnection {
    pub from_instance: usize,
    pub from_room: u32,
    pub to_instance: usize,
    pub to_room: u32,
}

/// Result of merging multiple local ship/station instances.
pub struct CompositeWorldData {
    pub graph: RoomGraph,
    pub grid: Vec<Vec<TileType>>,
    pub width: usize,
    pub height: usize,
}

/// Merges generated ship/station instances into one global graph and tile grid.
///
/// The `UmbilicalConnection` inputs are assumed to refer to room IDs that
/// exist in the corresponding instance-local graphs.
pub fn merge_instances(
    instances: Vec<InstancePlacement>,
    umbilical_connections: Vec<UmbilicalConnection>,
) -> Result<CompositeWorldData, String> {
    if instances.is_empty() {
        return Err("cannot merge zero instances".to_string());
    }

    for (idx, instance) in instances.iter().enumerate() {
        if instance.offset_x < 0 || instance.offset_y < 0 {
            return Err(format!("instance {idx} has negative offset"));
        }

        if instance.grid.is_empty() || instance.grid[0].is_empty() {
            return Err(format!("instance {idx} has empty grid"));
        }
    }

    // Determine global grid dimensions.
    let mut max_global_x = 0i64;
    let mut max_global_y = 0i64;

    for instance in &instances {
        let local_h = instance.grid.len() as i64;
        let local_w = instance.grid[0].len() as i64;
        max_global_x = max_global_x.max(instance.offset_x as i64 + local_w);
        max_global_y = max_global_y.max(instance.offset_y as i64 + local_h);
    }

    let width = max_global_x as usize;
    let height = max_global_y as usize;

    if width == 0 || height == 0 {
        return Err("merged grid must have non-zero dimensions".to_string());
    }

    // Merge tile grids.
    let mut global_grid = vec![vec![TileType::Void; width]; height];

    for (_instance_idx, instance) in instances.iter().enumerate() {
        let local_h = instance.grid.len();
        let local_w = instance.grid[0].len();

        for local_y in 0..local_h {
            for local_x in 0..local_w {
                let tile = instance.grid[local_y][local_x];
                if tile == TileType::Void {
                    continue;
                }

                let global_x = (instance.offset_x as i64 + local_x as i64) as usize;
                let global_y = (instance.offset_y as i64 + local_y as i64) as usize;

                if global_x >= width || global_y >= height {
                    return Err(format!(
                        "instance tile ({local_x},{local_y}) falls outside merged grid"
                    ));
                }

                if global_grid[global_y][global_x] != TileType::Void {
                    return Err(format!(
                        "tile overlap during merge at ({global_x},{global_y})"
                    ));
                }

                global_grid[global_y][global_x] = tile;
            }
        }
    }

    // Merge room graphs with globally unique room IDs.
    let mut merged_graph = RoomGraph::new();
    let mut next_room_id: u32 = 0;
    let mut room_mappings: Vec<HashMap<u32, u32>> = Vec::with_capacity(instances.len());

    for instance in instances.iter() {
        let mut mapping = HashMap::new();

        for node in instance.graph.nodes() {
            let new_id = next_room_id;
            next_room_id += 1;

            mapping.insert(node.id, new_id);

            let offset_rect = Rect::new(
                node.shape.x + instance.offset_x as f32,
                node.shape.y + instance.offset_y as f32,
                node.shape.width,
                node.shape.height,
            );

            merged_graph.add_node(RoomNode {
                id: new_id,
                room_type: node.room_type,
                shape: offset_rect,
                system_flags: node.system_flags,
            });
        }

        room_mappings.push(mapping);
    }

    // Add all internal instance edges.
    for (instance_idx, instance) in instances.iter().enumerate() {
        let mapping = &room_mappings[instance_idx];

        for edge in instance.graph.edges() {
            let from = mapping.get(&edge.from).copied().ok_or_else(|| {
                format!("internal edge references unknown from-room {}", edge.from)
            })?;
            let to = mapping.get(&edge.to).copied().ok_or_else(|| {
                format!("internal edge references unknown to-room {}", edge.to)
            })?;

            merged_graph.add_edge(RoomEdge {
                kind: edge.kind,
                from,
                to,
                location: edge.location,
            });
        }
    }

    // Add umbilical connections.
    for connection in &umbilical_connections {
        let from_mapping = room_mappings
            .get(connection.from_instance)
            .ok_or_else(|| format!("unknown from_instance {}", connection.from_instance))?;
        let to_mapping = room_mappings
            .get(connection.to_instance)
            .ok_or_else(|| format!("unknown to_instance {}", connection.to_instance))?;

        let from = from_mapping.get(&connection.from_room).copied().ok_or_else(|| {
            format!(
                "unknown from_room {} in instance {}",
                connection.from_room, connection.from_instance
            )
        })?;
        let to = to_mapping.get(&connection.to_room).copied().ok_or_else(|| {
            format!(
                "unknown to_room {} in instance {}",
                connection.to_room, connection.to_instance
            )
        })?;

        merged_graph.add_edge(RoomEdge {
            kind: EdgeKind::Umbilical,
            from,
            to,
            // For this first composite slice, the exact edge placement is
            // not needed for connectivity or system graph derivation.
            location: EdgePlacement::North,
        });
    }

    Ok(CompositeWorldData {
        graph: merged_graph,
        grid: global_grid,
        width,
        height,
    })
}
