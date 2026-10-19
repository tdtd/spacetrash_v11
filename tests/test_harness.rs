use spacetrash_v11::components::CircleCollider;
use spacetrash_v11::testing::{run_n_ticks, spawn_actor, test_app};

#[test]
fn test_harness_initializes_and_spawns_actor() {
    let mut app = test_app();

    let actor = spawn_actor(&mut app, 0.1667);
    assert!(app.world.get_entity(actor).is_some());

    let collider = app.world.get::<CircleCollider>(actor).unwrap();
    assert!((collider.radius - 0.1667).abs() < f32::EPSILON);

    run_n_ticks(&mut app, 1);
}
