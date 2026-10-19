use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::procgen::scaffold::RoomGraph;

pub mod ship_template;
pub mod scenario;
pub mod procgen;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Victory,
}

#[derive(Resource)]
pub struct PhysicsConfig {
    pub fixed_timestep: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub damping: f32,
    pub angular_acceleration: f32,
    pub angular_max_speed: f32,
    pub angular_damping: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            fixed_timestep: 1.0 / 60.0,
            max_speed: 4.0,
            acceleration: 20.0,
            damping: 10.0,
            angular_acceleration: 12.0,
            angular_max_speed: 8.0,
            angular_damping: 6.0,
        }
    }
}

#[derive(Resource)]
pub struct InputConfig {
    pub mouse_sensitivity: f32,
    pub gamepad_deadzone: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.1,
            gamepad_deadzone: 0.2,
        }
    }
}

#[derive(Resource, Default)]
pub struct EditorMode {
    pub active: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTool {
    #[default]
    Select,
    Room,
    Door,
    Delete,
    Move,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiegeticView {
    #[default]
    FirstPerson,
    CommandLine,
    Schematic,
    ConfigView,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SchematicLayer {
    #[default]
    Structural,
    Electrical,
    Fluidics,
    Atmospherics,
    Thermal,
    Computer,
    Research,
    Docking,
    Sensors,
    Reserved,
}

#[derive(Resource, Default)]
pub struct SchematicLayers {
    pub active: HashSet<SchematicLayer>,
}

#[derive(Resource, Default)]
pub struct FocusedEntity {
    pub entity: Option<Entity>,
}

#[derive(Resource)]
pub struct CurrentRoomGraph {
    pub graph: RoomGraph,
    pub origin_x: f32,
    pub origin_z: f32,
    pub width: usize,
    pub height: usize,
}

/// Derived room-level system connectivity graphs.
///
/// Keys are `RoomId` values. Edges are undirected.
#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentSystemGraphs {
    pub electrical: HashMap<u32, Vec<u32>>,
    pub fluidics: HashMap<u32, Vec<u32>>,
    pub atmospherics: HashMap<u32, Vec<u32>>,
    pub thermal: HashMap<u32, Vec<u32>>,
    pub computer: HashMap<u32, Vec<u32>>,
}
