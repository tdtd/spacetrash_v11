use std::collections::HashMap;

use bevy::prelude::*;

use super::ship_template::{ShipTemplate, Scenario, ShipInstanceInit};

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

/// A resource that stores all selectable scenarios, keyed by ID.
#[derive(Resource, Default)]
pub struct ScenarioCatalog {
    pub scenarios: HashMap<String, Scenario>,
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

fn make_ship_template(
    id: impl Into<String>,
    name: impl Into<String>,
    seed: u64,
    grid_cols: usize,
    grid_rows: usize,
    min_room_size: f32,
    max_room_size: f32,
) -> ShipTemplate {
    ShipTemplate {
        id: id.into(),
        name: name.into(),
        seed,
        room_count: grid_cols * grid_rows,
        min_room_size,
        max_room_size,
        door_width: 1.0,
        wall_thickness: 0.2,
        wall_height: 2.0,
        manual_rooms: vec![],
        door_overrides: vec![],
        docking_ports: vec![],
        grid_cols,
        grid_rows,
        corridor_width: 1.0,
    }
}

/// Small fast scout: long, narrow profile.
pub fn scout_ship_template() -> ShipTemplate {
    make_ship_template(
        "scout_ship",
        "Scout Ship",
        2024101901,
        2,
        1,
        4.0,
        4.0,
    )
}

/// A bulkier freighter with varied room sizes.
pub fn freighter_ship_template() -> ShipTemplate {
    make_ship_template(
        "freighter",
        "Freighter",
        2024101902,
        3,
        2,
        4.0,
        8.0,
    )
}

/// A compact corvette-style ship.
pub fn corvette_ship_template() -> ShipTemplate {
    make_ship_template(
        "corvette",
        "Corvette",
        2024101903,
        2,
        2,
        4.0,
        6.0,
    )
}

/// A long salvage barge.
pub fn salvage_barge_template() -> ShipTemplate {
    make_ship_template(
        "salvage_barge",
        "Salvage Barge",
        2024101904,
        4,
        1,
        3.0,
        5.0,
    )
}

/// A larger central station hub.
pub fn station_hub_template() -> ShipTemplate {
    make_ship_template(
        "station_hub",
        "Station Hub",
        2024101905,
        3,
        3,
        5.0,
        7.0,
    )
}

/// Creates the one-ship scenario.
pub fn one_ship_scenario() -> Scenario {
    Scenario {
        id: "one_ship".to_string(),
        name: "One Ship".to_string(),
        ships: vec![ShipInstanceInit {
            template_id: "freighter".to_string(),
            position: [0.0, 0.0],
            rotation: 0.0,
        }],
        docking_links: vec![],
        world_state: None,
    }
}

/// Creates the default scenario used by `Start Game`.
pub fn default_scenario() -> Scenario {
    one_ship_scenario()
}

/// Creates a two-ship scenario with distinct ship types.
pub fn two_ship_scenario() -> Scenario {
    Scenario {
        id: "two_ships".to_string(),
        name: "Two Ships".to_string(),
        ships: vec![
            ShipInstanceInit {
                template_id: "scout_ship".to_string(),
                position: [-14.0, 0.0],
                rotation: 0.15,
            },
            ShipInstanceInit {
                template_id: "freighter".to_string(),
                position: [14.0, 0.0],
                rotation: -0.12,
            },
        ],
        docking_links: vec![],
        world_state: None,
    }
}

/// Creates a four-ship plus central station scenario.
///
/// Each surrounding ship uses a different template, and the station uses the
/// larger `station_hub` template.
pub fn four_ship_station_scenario() -> Scenario {
    Scenario {
        id: "four_ship_station".to_string(),
        name: "Four Ships + Station".to_string(),
        ships: vec![
            ShipInstanceInit {
                template_id: "station_hub".to_string(),
                position: [0.0, 0.0],
                rotation: 0.0,
            },
            ShipInstanceInit {
                template_id: "corvette".to_string(),
                position: [0.0, 24.0],
                rotation: 0.0,
            },
            ShipInstanceInit {
                template_id: "scout_ship".to_string(),
                position: [24.0, 0.0],
                rotation: std::f32::consts::FRAC_PI_2,
            },
            ShipInstanceInit {
                template_id: "salvage_barge".to_string(),
                position: [0.0, -24.0],
                rotation: std::f32::consts::PI,
            },
            ShipInstanceInit {
                template_id: "freighter".to_string(),
                position: [-24.0, 0.0],
                rotation: -std::f32::consts::FRAC_PI_2,
            },
        ],
        docking_links: vec![],
        world_state: None,
    }
}

/// Creates the built-in scenario catalog for the main menu.
pub fn builtin_scenario_catalog() -> ScenarioCatalog {
    let mut scenarios = HashMap::new();
    scenarios.insert("one_ship".to_string(), one_ship_scenario());
    scenarios.insert("two_ships".to_string(), two_ship_scenario());
    scenarios.insert(
        "four_ship_station".to_string(),
        four_ship_station_scenario(),
    );

    ScenarioCatalog { scenarios }
}

/// Creates the built-in ship-template catalog used by the runtime.
pub fn builtin_ship_template_catalog() -> HashMap<String, ShipTemplate> {
    let mut templates = HashMap::new();
    templates.insert("test_ship".to_string(), default_ship_template());
    templates.insert("scout_ship".to_string(), scout_ship_template());
    templates.insert("freighter".to_string(), freighter_ship_template());
    templates.insert("corvette".to_string(), corvette_ship_template());
    templates.insert("salvage_barge".to_string(), salvage_barge_template());
    templates.insert("station_hub".to_string(), station_hub_template());

    templates
}
