use bevy::prelude::*;

use crate::components::{AiBehaviorState, BehaviorDirective, Rotation, Velocity};

/// Copies an actor's `BehaviorDirective` into its current `AiBehaviorState`.
///
/// This is the first, deliberately simple step toward player-controlled
/// behavior changes. A future version will add situation-driven arbitration
/// when no directive is present, while low-health flee can still override
/// if needed.
pub fn behavior_arbitration_system(
    mut query: Query<(Option<&BehaviorDirective>, &mut AiBehaviorState)>,
) {
    for (directive, mut state) in query.iter_mut() {
        let Some(directive) = directive else {
            continue;
        };

        let next = match directive {
            BehaviorDirective::None => continue,
            BehaviorDirective::Patrol => AiBehaviorState::Patrol,
            BehaviorDirective::Guard => AiBehaviorState::Guard,
            BehaviorDirective::Aggressive => AiBehaviorState::Aggressive,
            BehaviorDirective::Flee => AiBehaviorState::Flee,
            BehaviorDirective::Scavenge => AiBehaviorState::Scavenge,
            BehaviorDirective::FollowLeader => AiBehaviorState::FollowLeader,
            BehaviorDirective::Passive => AiBehaviorState::Passive,
        };

        if *state != next {
            *state = next;
        }
    }
}

/// Crude behavior-driven movement.
///
/// For now this only exists to make actors visibly move during play and to
/// give TDD tests a stable point for asserting that an active behavior state
/// produces a non-zero velocity.
///
/// This system is deliberately simple and deterministic for a given elapsed
/// time. It will later be replaced by per-behavior steering systems that use
/// the sensor grid and target selection.
pub fn behavior_movement_system(
    time: Res<Time>,
    mut query: Query<(&AiBehaviorState, &mut Velocity, &Rotation)>,
) {
    let t = time.elapsed_seconds();

    for (state, mut velocity, rotation) in query.iter_mut() {
        let desired_velocity = match *state {
            AiBehaviorState::Patrol => {
                // Move around in a simple circle.
                let angle = t * 1.5;
                Vec3::new(angle.sin(), 0.0, angle.cos()) * 1.0
            }
            AiBehaviorState::Guard => Vec3::ZERO,
            AiBehaviorState::Aggressive => {
                let forward = rotation_to_forward(rotation.yaw);
                forward * 2.0
            }
            AiBehaviorState::Flee => {
                let forward = rotation_to_forward(rotation.yaw);
                -forward * 2.0
            }
            AiBehaviorState::Scavenge => {
                let angle = t * 0.8;
                Vec3::new(angle.cos(), 0.0, angle.sin()) * 1.0
            }
            AiBehaviorState::FollowLeader => {
                let forward = rotation_to_forward(rotation.yaw);
                forward * 1.5
            }
            AiBehaviorState::Passive => Vec3::ZERO,
        };

        velocity.0 = desired_velocity;
    }
}

fn rotation_to_forward(yaw: f32) -> Vec3 {
    Vec3::new(yaw.sin(), 0.0, -yaw.cos())
}
