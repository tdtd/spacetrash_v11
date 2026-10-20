use std::time::Duration;

use bevy::prelude::*;
use bevy::time::{TimePlugin, TimeUpdateStrategy};

use crate::components::{AiBehaviorState, Rotation, Velocity};
use crate::resources::PhysicsConfig;
use crate::resources::SimulationSpeed;
use crate::systems::behavior::behavior_arbitration_system;
use crate::systems::movement_system;
use crate::systems::set_pieces::set_piece_thermal_system;

use super::config::ArenaConfig;
use super::event::{ActorState, ArenaEvent, ArenaRunResult, ArenaSnapshot};
use super::session::SimulationSession;
use super::world_builder::build_arena_world;

/// Builds a minimal deterministic Bevy `App` for headless arena episodes.
///
/// Uses `MinimalPlugins` with `TimePlugin` disabled, matching the project's
/// existing deterministic test harness.
pub fn build_arena_app(config: &ArenaConfig) -> App {
    let mut app = App::new();

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        PhysicsConfig::default().fixed_timestep,
    )))
    .insert_resource(PhysicsConfig::default())
    .insert_resource(SimulationSpeed::default())
    .init_resource::<Time>()
    .insert_resource(SimulationSession::new(config.clone()));

    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());

    // Spawn ship geometry, resources, and actors from config.
    build_arena_world(&mut app.world, config);

    // Behavior arbitration, movement, thermal transfer, then structured snapshot capture.
    app.add_systems(
        Update,
        (
            behavior_arbitration_system,
            movement_system,
            set_piece_thermal_system,
            record_arena_snapshot,
        )
            .chain(),
    );

    app
}

/// Records a deterministic snapshot for every actor after each tick.
fn record_arena_snapshot(
    mut session: ResMut<SimulationSession>,
    actors: Query<(
        Entity,
        &Transform,
        &Velocity,
        &Rotation,
        Option<&AiBehaviorState>,
    )>,
) {
    let mut states: Vec<ActorState> = actors
        .iter()
        .map(
            |(entity, transform, velocity, rotation, behavior)| ActorState {
                entity_index: entity.index(),
                translation: transform.translation,
                velocity: velocity.0,
                yaw: rotation.yaw,
                behavior: behavior.copied().unwrap_or_default(),
            },
        )
        .collect();

    states.sort_by_key(|state| state.entity_index);

    let tick = session.tick;
    session.event_log.events.push(ArenaEvent::TickSnapshot {
        tick,
        actors: states,
    });
}

/// Runs one headless arena episode.
///
/// The result contains the final snapshot and the full structured event log.
pub fn run_arena(config: &ArenaConfig) -> ArenaRunResult {
    let mut app = build_arena_app(config);

    let dt = Duration::from_secs_f32(PhysicsConfig::default().fixed_timestep);

    for tick in 0..config.tick_count {
        app.world.resource_mut::<SimulationSession>().tick = tick;
        app.world.resource_mut::<Time>().advance_by(dt);
        app.update();
    }

    let event_log = app.world.resource::<SimulationSession>().event_log.clone();

    let final_snapshot = event_log
        .events
        .last()
        .and_then(|event| match event {
            ArenaEvent::TickSnapshot { actors, .. } => Some(ArenaSnapshot {
                actors: actors.clone(),
            }),
        })
        .unwrap_or_default();

    ArenaRunResult {
        final_snapshot,
        event_log,
    }
}
