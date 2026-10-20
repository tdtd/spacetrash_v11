use bevy::prelude::*;
use spacetrash_v11::arena::{run_arena, ArenaActorSpawn, ArenaConfig};

#[test]
fn same_arena_config_is_byte_for_byte_repeatable() {
    let config = ArenaConfig {
        seed: 1337,
        tick_count: 300,
        actors: vec![
            ArenaActorSpawn {
                translation: Vec3::new(0.0, 0.25, 0.0),
                velocity: Vec3::new(0.8, 0.0, 0.3),
                radius: 0.1667,
                yaw: 0.0,
                angular_velocity: 0.1,
            },
            ArenaActorSpawn {
                translation: Vec3::new(2.0, 0.25, -1.0),
                velocity: Vec3::new(-0.4, 0.0, 0.7),
                radius: 0.1667,
                yaw: 1.0,
                angular_velocity: -0.2,
            },
        ],
        ship: None,
    };

    let first = run_arena(&config);
    let second = run_arena(&config);

    assert_eq!(first.final_snapshot, second.final_snapshot);
    assert_eq!(first.event_log, second.event_log);
}
