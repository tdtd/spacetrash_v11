use std::collections::HashMap;

use bevy::prelude::*;

use super::ship_template::{ShipTemplate, Scenario};

/// A resource that stores all available ship templates, keyed by ID.
#[derive(Resource, Default)]
pub struct ShipTemplateLibrary {
    pub templates: HashMap<String, ShipTemplate>,
}

/// A resource that stores the currently active scenario (if any).
#[derive(Resource, Default)]
pub struct ActiveScenario {
    pub scenario: Option<Scenario>,
}

/// Helper function to create a default ship template for testing.
pub fn default_ship_template() -> ShipTemplate {
    ShipTemplate {
        id: "test_ship".to_string(),
        name: "Test Ship".to_string(),
        seed: 12345,
        room_count: 6,           // rooms in the grid (grid_cols * grid_rows)
        min_room_size: 4.0,
        max_room_size: 10.0,
        door_width: 1.0,         // 1-tile door openings
        wall_thickness: 0.2,
        wall_height: 2.0,
        manual_rooms: vec![],
        door_overrides: vec![],
        docking_ports: vec![],
        grid_cols: 3,
        grid_rows: 2,
        corridor_width: 1.0,     // 1-tile deadzone/corridor between rooms
    }
}
