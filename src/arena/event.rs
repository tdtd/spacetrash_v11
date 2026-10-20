use bevy::prelude::*;

use crate::components::AiBehaviorState;

/// A stable snapshot of one actor at one tick.
#[derive(Clone, Debug, PartialEq)]
pub struct ActorState {
    pub entity_index: u32,
    pub translation: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub behavior: AiBehaviorState,
}

/// One structured arena event.
///
/// Phase 1 only logs full tick snapshots. Later events such as
/// `CollisionResolved`, `SensorDetection`, and `FsaTransition` will be added.
#[derive(Clone, Debug, PartialEq)]
pub enum ArenaEvent {
    TickSnapshot {
        tick: u64,
        actors: Vec<ActorState>,
    },
}

/// Ordered event log for an arena episode.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct ArenaEventLog {
    pub events: Vec<ArenaEvent>,
}

/// Current fixed-tick counter.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct ArenaClock {
    pub tick: u64,
}

/// Final world snapshot after an arena episode.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArenaSnapshot {
    pub actors: Vec<ActorState>,
}

/// Result of running one arena episode.
#[derive(Clone, Debug, PartialEq)]
pub struct ArenaRunResult {
    pub final_snapshot: ArenaSnapshot,
    pub event_log: ArenaEventLog,
}
