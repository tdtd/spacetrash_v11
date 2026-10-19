use bevy::prelude::*;

use spacetrash_v11::components::ActorSize;
use spacetrash_v11::systems::movement_system;
use spacetrash_v11::testing::{run_n_ticks, spawn_actor_with_size, spawn_static_wall, test_app};

#[test]
fn actor_size_radius_mapping() {
    assert_eq!(ActorSize::Small.collider_radius(), None);

    let standard = ActorSize::Standard.collider_radius().unwrap();
    assert!((standard - (1.0 / 6.0)).abs() < 1e-6);

    let large = ActorSize::Large.collider_radius().unwrap();
    assert!((large - (1.0 / 3.0)).abs() < 1e-6);
}

#[test]
fn small_actor_ignores_static_collision() {
    let mut app = test_app();
    app.add_systems(Update, movement_system);

    let actor = spawn_actor_with_size(&mut app, 0.1667, ActorSize::Small);
    let _wall = spawn_static_wall(&mut app, Vec3::new(1.0, 1.0, 0.5));

    {
        let mut transform = app.world.get_mut::<Transform>(actor).unwrap();
        transform.translation = Vec3::new(1.05, 0.25, 0.0);
    }

    run_n_ticks(&mut app, 1);

    let transform = app.world.get::<Transform>(actor).unwrap();
    assert!(
        (transform.translation.x - 1.05).abs() < 1e-5,
        "Small actor should not be pushed by static wall"
    );
}

#[test]
fn standard_actor_collides_with_static_collision() {
    let mut app = test_app();
    app.add_systems(Update, movement_system);

    let actor = spawn_actor_with_size(&mut app, 0.1667, ActorSize::Standard);
    let _wall = spawn_static_wall(&mut app, Vec3::new(1.0, 1.0, 0.5));

    {
        let mut transform = app.world.get_mut::<Transform>(actor).unwrap();
        transform.translation = Vec3::new(1.05, 0.25, 0.0);
    }

    run_n_ticks(&mut app, 1);

    let transform = app.world.get::<Transform>(actor).unwrap();
    assert!(
        (transform.translation.x - 1.1667).abs() < 1e-5,
        "Standard actor should be pushed to wall edge + radius"
    );
}

#[test]
fn small_actor_ignores_dynamic_collision() {
    let mut app = test_app();
    app.add_systems(Update, movement_system);

    let small = spawn_actor_with_size(&mut app, 0.1667, ActorSize::Small);
    let standard = spawn_actor_with_size(&mut app, 0.1667, ActorSize::Standard);

    run_n_ticks(&mut app, 1);

    let pos_small = app.world.get::<Transform>(small).unwrap().translation;
    let pos_standard = app.world.get::<Transform>(standard).unwrap().translation;

    assert!(
        pos_small.x.abs() < 1e-5 && pos_small.z.abs() < 1e-5,
        "Small actor should remain at origin"
    );
    assert!(
        pos_standard.x.abs() < 1e-5 && pos_standard.z.abs() < 1e-5,
        "Standard actor should also remain at origin when paired with Small"
    );
}
