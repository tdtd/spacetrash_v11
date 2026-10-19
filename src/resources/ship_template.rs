use serde::{Deserialize, Serialize};

/// A 2D position in world space (XZ plane). Y is always 0 for gameplay.
pub type Position2 = [f32; 2];

/// Definition for a docking port on a ship template.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockingPortDef {
    pub local_position: Position2,
    pub facing: Facing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Facing {
    North,
    South,
    East,
    West,
}

/// Procedural generation parameters for a ship.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShipTemplate {
    pub id: String,
    pub name: String,
    pub seed: u64,
    /// Number of rooms to generate (procedural part).
    pub room_count: usize,
    /// Minimum room width and depth in world units.
    pub min_room_size: f32,
    /// Maximum room width and depth in world units.
    pub max_room_size: f32,
    /// Width of door openings in world units (1 tile = 1.0).
    pub door_width: f32,
    /// Wall thickness (used for collision and visual).
    pub wall_thickness: f32,
    /// Wall height (visual only).
    pub wall_height: f32,
    /// Number of room columns in the generated grid.
    pub grid_cols: usize,
    /// Number of room rows in the generated grid.
    pub grid_rows: usize,
    /// Width of the corridors between rooms.
    pub corridor_width: f32,
    /// Manual room overrides. Each defines a rectangle in local grid coordinates.
    pub manual_rooms: Vec<ManualRoom>,
    /// Manual door additions/removals. (Can be extended later.)
    pub door_overrides: Vec<DoorOverride>,
    /// Docking port placements for this ship.
    pub docking_ports: Vec<DockingPortDef>,
}

/// A manually placed room (overrides or extends procedural layout).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManualRoom {
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub depth: f32,
}

/// Instructions to add or remove a door at a given wall position.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoorOverride {
    pub x: f32,
    pub z: f32,
    /// True = add a door, false = remove a door.
    pub add: bool,
}

/// A scenario defining the initial (or saved) world composition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub ships: Vec<ShipInstanceInit>,
    pub docking_links: Vec<DockingLink>,
    pub world_state: Option<WorldState>,
}

/// Initialisation data for a ship instance in a scenario.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShipInstanceInit {
    pub template_id: String,
    pub position: Position2, // world position of ship origin
    pub rotation: f32,       // rotation around Y axis in radians
}

/// A docking link between two ship instances.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockingLink {
    pub ship_a: usize,   // index into `ships`
    pub port_a: usize,   // index into ship_a's template.docking_ports
    pub ship_b: usize,
    pub port_b: usize,
}

/// Placeholder for full world save data. Will be expanded later.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldState {
    // Will contain serialised entities, system states, etc.
}
