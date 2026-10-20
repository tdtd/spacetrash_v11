use bevy::prelude::*;

use super::config::ArenaConfig;
use super::event::{ArenaEventLog, ArenaSnapshot};

/// Shared deterministic simulation lifecycle used by both headless
/// runners and the future spectator mode.
#[derive(Resource, Clone, Debug)]
pub struct SimulationSession {
    pub config: ArenaConfig,
    pub tick: u64,
    pub state: SimulationState,
    pub speed: SimulationSpeed,
    pub event_log: ArenaEventLog,
    pub final_snapshot: Option<ArenaSnapshot>,
}

impl SimulationSession {
    pub fn new(config: ArenaConfig) -> Self {
        Self {
            config,
            tick: 0,
            state: SimulationState::Stopped,
            speed: SimulationSpeed::Paused,
            event_log: ArenaEventLog::default(),
            final_snapshot: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    Stopped,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationSpeed {
    Paused,
    Normal,
    Fast,
}
