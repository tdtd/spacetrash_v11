use bevy::prelude::*;

use crate::arena::{
    run_arena, ActorState, ArenaActorSpawn, ArenaConfig, ArenaEvent,
};
use crate::components::{GameplayEntity, SpectatorActor};
use crate::resources::{SpectatorMode, SpectatorReplay};

/// Default spectator replay configuration.
///
/// This creates a small headless arena with three actors and no ship,
/// just to demonstrate playback.
const SPECTATOR_ARENA_CONFIG: fn() -> ArenaConfig = || ArenaConfig {
    seed: 123,
    tick_count: 240,
    actors: vec![
        ArenaActorSpawn {
            translation: Vec3::new(0.0, 0.25, 0.0),
            velocity: Vec3::new(1.0, 0.0, 0.0),
            radius: 0.1667,
            yaw: 0.0,
            angular_velocity: 0.0,
        },
        ArenaActorSpawn {
            translation: Vec3::new(2.0, 0.25, 0.0),
            velocity: Vec3::new(-0.5, 0.0, 0.5),
            radius: 0.1667,
            yaw: 1.0,
            angular_velocity: 0.1,
        },
        ArenaActorSpawn {
            translation: Vec3::new(-1.0, 0.25, 1.0),
            velocity: Vec3::new(0.2, 0.0, -0.7),
            radius: 0.1667,
            yaw: 2.0,
            angular_velocity: -0.05,
        },
    ],
    ship: None,
};

/// System that runs while spectator mode is active.
///
/// It generates the replay data once, hides the normal gameplay world,
/// spawns simple visual actor cubes, and updates their transforms from
/// the recorded snapshots. When spectator mode ends, it restores visibility
/// and despawns the visual actors.
pub fn spectator_playback_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spectator_mode: Res<SpectatorMode>,
    mut replay: ResMut<SpectatorReplay>,
    time: Res<Time>,
    mut spectator_actors: Query<(Entity, &SpectatorActor, &mut Transform)>,
    mut gameplay_entities: Query<&mut Visibility, With<GameplayEntity>>,
) {
    if !spectator_mode.active {
        // Restore normal gameplay visibility and despawn spectator actors.
        for mut visibility in gameplay_entities.iter_mut() {
            *visibility = Visibility::Inherited;
        }

        for (entity, _, _) in spectator_actors.iter() {
            commands.entity(entity).despawn_recursive();
        }

        return;
    }

    // Hide normal gameplay entities while spectator is active.
    for mut visibility in gameplay_entities.iter_mut() {
        *visibility = Visibility::Hidden;
    }

    // Generate replay data and spawn visual actors on first activation.
    if replay.event_log.events.is_empty() {
        generate_spectator_replay(&mut replay, &mut commands, &mut meshes, &mut materials);
    }

    // Advance playback if playing.
    if replay.is_playing && !replay.event_log.events.is_empty() {
        let advance = time.delta_seconds() * replay.playback_speed;
        replay.current_tick = replay.current_tick.saturating_add(advance as u32);
        let total_ticks = replay.event_log.events.len() as u32;
        if replay.current_tick >= total_ticks {
            replay.current_tick = 0;
        }
    }

    // Apply current snapshot to visual actors.
    let snapshot: Option<Vec<ActorState>> = replay
        .event_log
        .events
        .get(replay.current_tick as usize)
        .and_then(|event| match event {
            ArenaEvent::TickSnapshot { actors, .. } => Some(actors.clone()),
        });

    if let Some(actors) = snapshot {
        for actor_state in actors {
            for (_entity, spectator_actor, mut transform) in spectator_actors.iter_mut() {
                if spectator_actor.entity_index == actor_state.entity_index {
                    transform.translation = actor_state.translation;
                    break;
                }
            }
        }
    }
}

/// Generates replay data by running a headless arena once and spawns
/// one cube per actor in the first snapshot.
fn generate_spectator_replay(
    replay: &mut SpectatorReplay,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let config = SPECTATOR_ARENA_CONFIG();
    let result = run_arena(&config);

    replay.event_log = result.event_log.clone();
    replay.current_tick = 0;
    replay.playback_speed = 1.0;
    replay.is_playing = true;

    // Spawn visual actors based on the first snapshot.
    let first_snapshot = replay
        .event_log
        .events
        .first()
        .and_then(|event| match event {
            ArenaEvent::TickSnapshot { actors, .. } => Some(actors.clone()),
        });

    let Some(actors) = first_snapshot else {
        return;
    };

    let mesh = meshes.add(Mesh::from(Cuboid::new(0.3, 0.3, 0.3)));
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.9, 0.5, 0.2),
        ..Default::default()
    });

    for actor_state in actors {
        commands.spawn((
            PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(actor_state.translation),
                ..Default::default()
            },
            SpectatorActor {
                entity_index: actor_state.entity_index,
            },
        ));
    }
}
