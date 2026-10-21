use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::arena::event::ArenaEventLog;
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

/// Developer spectator mode.
///
/// This is the headful counterpart to the headless `run_arena` runner.
/// It is not a player-facing mechanic; it is used to inspect deterministic
/// arena episodes and generated maps.
#[derive(Resource, Default)]
pub struct SpectatorMode {
    pub active: bool,
}

/// Replay state used when spectator mode is active.
#[derive(Resource, Default)]
pub struct SpectatorReplay {
    pub event_log: ArenaEventLog,
    pub current_tick: u32,
    pub playback_speed: f32,
    pub is_playing: bool,
}

/// Simulation speed multiplier.
///
/// This is a debug/out-of-game control exposed through the terminal
/// command line, not through the HUD.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct SimulationSpeed {
    pub scale: f32,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
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

/// Current terminal command input buffer and history.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct TerminalInputBuffer {
    pub current: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub saved_draft: String,
}

impl TerminalInputBuffer {
    /// Appends a non-empty command to history, skipping consecutive duplicates,
    /// and resets the history navigation cursor.
    pub fn push_history(&mut self, cmd: impl Into<String>) {
        let cmd = cmd.into();
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            self.history_index = None;
            self.saved_draft.clear();
            return;
        }

        if self.history.last().map(|s| s.as_str()) != Some(trimmed) {
            self.history.push(trimmed.to_string());
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
        self.history_index = None;
        self.saved_draft.clear();
    }

    /// Navigates backwards in command history (Arrow Up).
    pub fn navigate_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.saved_draft = self.current.clone();
                let last = self.history.len() - 1;
                self.history_index = Some(last);
                self.current = self.history[last].clone();
            }
            Some(idx) => {
                if idx > 0 {
                    let prev = idx - 1;
                    self.history_index = Some(prev);
                    self.current = self.history[prev].clone();
                }
            }
        }
    }

    /// Navigates forward in command history (Arrow Down), restoring any saved draft at the end.
    pub fn navigate_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx + 1 < self.history.len() {
                let next = idx + 1;
                self.history_index = Some(next);
                self.current = self.history[next].clone();
            } else {
                self.history_index = None;
                self.current = std::mem::take(&mut self.saved_draft);
            }
        }
    }
}

/// Ordered terminal output log.
///
/// Kept as a resource so it can be read by UI and used in headless tests
/// without depending on UI entities.
#[derive(Resource, Default)]
pub struct TerminalOutputLog {
    pub lines: Vec<String>,
}

impl TerminalOutputLog {
    pub fn push_line(&mut self, line: impl Into<String>) {
        let line = line.into();
        self.lines.push(line);
        while self.lines.len() > 200 {
            self.lines.remove(0);
        }
    }
}
