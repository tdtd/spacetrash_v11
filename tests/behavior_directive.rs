use bevy::prelude::*;
use spacetrash_v11::components::{AiBehaviorState, BehaviorDirective, Drone, Rotation, Velocity};
use spacetrash_v11::systems::behavior::{behavior_arbitration_system, behavior_movement_system};
use spacetrash_v11::testing::{run_n_ticks, test_app};

#[test]
fn behavior_directive_changes_drone_ai_state() {
    let mut app = test_app();

    let drone = app
        .world
        .spawn((
            Drone { index: 1 },
            BehaviorDirective::Guard,
            AiBehaviorState::default(),
        ))
        .id();

    app.add_systems(Update, behavior_arbitration_system);

    run_n_ticks(&mut app, 1);

    let state = app.world.get::<AiBehaviorState>(drone).unwrap();
    assert_eq!(*state, AiBehaviorState::Guard);
}

#[test]
fn behavior_directive_none_does_not_force_state() {
    let mut app = test_app();

    let drone = app
        .world
        .spawn((
            Drone { index: 2 },
            BehaviorDirective::None,
            AiBehaviorState::default(),
        ))
        .id();

    app.add_systems(Update, behavior_arbitration_system);

    run_n_ticks(&mut app, 1);

    let state = app.world.get::<AiBehaviorState>(drone).unwrap();
    assert_eq!(*state, AiBehaviorState::Passive);
}

#[test]
fn behavior_movement_system_sets_velocity_for_patrol() {
    let mut app = test_app();

    let actor = app
        .world
        .spawn((
            Rotation { yaw: 0.0 },
            Velocity::default(),
            AiBehaviorState::Patrol,
        ))
        .id();

    app.add_systems(Update, behavior_movement_system);

    run_n_ticks(&mut app, 1);

    let velocity = app.world.get::<Velocity>(actor).unwrap();
    assert!(velocity.0.length() > 0.0);
}
