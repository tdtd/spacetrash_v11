use bevy::prelude::*;

use spacetrash_v11::components::{AngularVelocity, Velocity};
use spacetrash_v11::resources::FocusedEntity;
use spacetrash_v11::systems::movement_system;
use spacetrash_v11::testing::{press_key, run_n_ticks, spawn_actor, test_app};

#[test]
fn test_player_input_accelerates_forward_negative_z() {
    let mut app = test_app();
    app.add_systems(Update, spacetrash_v11::systems::player_input_system);

    let actor = spawn_actor(&mut app, 0.1667);
    app.insert_resource(FocusedEntity { entity: Some(actor) });

    press_key(&mut app, KeyCode::KeyW);
    run_n_ticks(&mut app, 1);

    let velocity = app.world.get::<Velocity>(actor).expect("actor has Velocity");
    assert!(
        velocity.0.z < 0.0,
        "forward key should produce negative Z velocity, got {:?}",
        velocity.0
    );
    assert!(
        velocity.0.x.abs() < 1e-6,
        "no strafe, x should be near zero, got {:?}",
        velocity.0
    );
}

#[test]
fn test_player_input_accelerates_rotation_left() {
    let mut app = test_app();
    app.add_systems(Update, spacetrash_v11::systems::player_input_system);

    let actor = spawn_actor(&mut app, 0.1667);
    app.insert_resource(FocusedEntity { entity: Some(actor) });

    press_key(&mut app, KeyCode::KeyQ);
    run_n_ticks(&mut app, 1);

    let angular_velocity = app
        .world
        .get::<AngularVelocity>(actor)
        .expect("actor has AngularVelocity");
    assert!(
        angular_velocity.yaw > 0.0,
        "left turn key should produce positive yaw rate, got {}",
        angular_velocity.yaw
    );
}

#[test]
fn two_overlapping_actors_get_separated() {
    let mut app = test_app();
    app.add_systems(Update, movement_system);

    // Spawn two standard actors at the same position (origin).
    let a = spawn_actor(&mut app, 0.1667);
    let b = spawn_actor(&mut app, 0.1667);

    // Ensure both have zero velocity.
    {
        let mut vel_a = app.world.get_mut::<Velocity>(a).unwrap();
        vel_a.0 = Vec3::ZERO;
        let mut vel_b = app.world.get_mut::<Velocity>(b).unwrap();
        vel_b.0 = Vec3::ZERO;
    }

    run_n_ticks(&mut app, 1);

    let pos_a = app.world.get::<Transform>(a).unwrap().translation;
    let pos_b = app.world.get::<Transform>(b).unwrap().translation;

    let min_dist = 0.1667 + 0.1667;
    assert!(
        (pos_a - pos_b).length() >= min_dist - 1e-5,
        "actors should be separated by at least sum of radii, got pos_a {:?}, pos_b {:?}",
        pos_a, pos_b
    );
}
