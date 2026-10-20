use bevy::prelude::*;

/// Configuration for a headless arena episode.
#[derive(Clone, Debug, PartialEq)]
pub struct ArenaConfig {
    pub seed: u64,
    pub tick_count: u64,
    pub actors: Vec<ArenaActorSpawn>,
    /// Optional ship/level generation config.
    pub ship: Option<ArenaShipConfig>,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            tick_count: 0,
            actors: Vec::new(),
            ship: None,
        }
    }
}

/// Explicit spawn data for a single arena actor.
///
/// This will later be replaced/augmented by upgrade-driven actor assembly.
/// For Phase 2 it remains intentionally simple and deterministic.
#[derive(Clone, Debug, PartialEq)]
pub struct ArenaActorSpawn {
    pub translation: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub angular_velocity: f32,
}

/// Deterministic ship-level generation parameters for arena episodes.
///
/// Kept intentionally independent from `ShipTemplate` so the headless
/// arena does not require render assets or template libraries.
#[derive(Clone, Debug, PartialEq)]
pub struct ArenaShipConfig {
    pub grid_cols: usize,
    pub grid_rows: usize,
    pub min_room_size: f32,
    pub max_room_size: f32,
    pub corridor_width: f32,
    pub wall_thickness: f32,
    pub wall_height: f32,
    pub seed: u64,
}

impl Default for ArenaShipConfig {
    fn default() -> Self {
        Self {
            grid_cols: 3,
            grid_rows: 2,
            min_room_size: 4.0,
            max_room_size: 10.0,
            corridor_width: 1.0,
            wall_thickness: 0.2,
            wall_height: 2.0,
            seed: 12345,
        }
    }
}
