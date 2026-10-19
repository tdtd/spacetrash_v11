use crate::procgen::scaffold::{RoomGraph, RoomNode, SystemFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerKind {
    Physical,
    Electrical,
    Fluidic,
    Atmospheric,
    Thermal,
    Computer,
    Docking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerTile {
    Empty,
    Floor,
    Wall,
    Door,
    Conduit,
    Terminal,
    Port,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRequirement {
    pub layer: LayerKind,
    pub coord: TileCoord,
    pub required_tile: LayerTile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathRequirement {
    pub layer: LayerKind,
    pub from: TileCoord,
    pub to: TileCoord,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphConstraints {
    pub cell_requirements: Vec<CellRequirement>,
    pub path_requirements: Vec<PathRequirement>,
}

/// Derives layer constraints from a room skeleton graph.
///
/// Constraint rules:
/// - Each room contributes a `Terminal` cell for every system flag introduced by
///   its `RoomType`.
/// - Each graph edge creates a path requirement for every layer shared by both
///   connected rooms.
pub fn derive_graph_constraints(graph: &RoomGraph) -> GraphConstraints {
    let mut constraints = GraphConstraints::default();

    for room in graph.nodes() {
        let center = room_center(room);

        for layer in flags_to_layers(room.system_flags) {
            constraints.cell_requirements.push(CellRequirement {
                layer,
                coord: center,
                required_tile: LayerTile::Terminal,
            });
        }
    }

    for edge in graph.edges() {
        let Some(from_room) = graph.get_node(edge.from) else {
            continue;
        };
        let Some(to_room) = graph.get_node(edge.to) else {
            continue;
        };

        let from_layers = flags_to_layers(from_room.system_flags);
        let to_layers = flags_to_layers(to_room.system_flags);

        for layer in from_layers {
            if to_layers.contains(&layer) {
                constraints.path_requirements.push(PathRequirement {
                    layer,
                    from: room_center(from_room),
                    to: room_center(to_room),
                });
            }
        }
    }

    constraints
}

fn flags_to_layers(flags: SystemFlags) -> Vec<LayerKind> {
    let mut layers = Vec::new();

    if flags.contains(SystemFlags::ELECTRICAL) {
        layers.push(LayerKind::Electrical);
    }
    if flags.contains(SystemFlags::THERMAL) {
        layers.push(LayerKind::Thermal);
    }
    if flags.contains(SystemFlags::ATMOSPHERICS) {
        layers.push(LayerKind::Atmospheric);
    }
    if flags.contains(SystemFlags::FLUIDICS) {
        layers.push(LayerKind::Fluidic);
    }
    if flags.contains(SystemFlags::COMPUTER) {
        layers.push(LayerKind::Computer);
    }

    layers
}

fn room_center(room: &RoomNode) -> TileCoord {
    TileCoord {
        x: (room.shape.x + room.shape.width * 0.5).floor() as i32,
        y: (room.shape.y + room.shape.height * 0.5).floor() as i32,
    }
}
