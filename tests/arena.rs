use bevy::prelude::*;
use spacetrash_v11::arena::{
    build_arena_app, generate_ship_data, reset_arena_world, run_arena, ArenaActorSpawn,
    ArenaConfig, ArenaShipConfig,
};
use spacetrash_v11::testing::map_assertions::MapAssert;

#[test]
fn arena_replay_is_deterministic() {
    let config = ArenaConfig {
        seed: 42,
        tick_count: 120,
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
        ],
        ship: None,
    };

    let first = run_arena(&config);
    let second = run_arena(&config);

    assert_eq!(first, second);
    assert_eq!(first.final_snapshot.actors.len(), 2);
    assert_eq!(first.event_log.events.len(), config.tick_count as usize);
}

#[test]
fn arena_ship_replay_is_deterministic() {
    let config = ArenaConfig {
        seed: 7,
        tick_count: 60,
        actors: vec![ArenaActorSpawn {
            translation: Vec3::new(0.0, 0.25, 0.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
            radius: 0.1667,
            yaw: 0.0,
            angular_velocity: 0.0,
        }],
        ship: Some(ArenaShipConfig {
            grid_cols: 2,
            grid_rows: 1,
            min_room_size: 4.0,
            max_room_size: 4.0,
            corridor_width: 1.0,
            wall_thickness: 0.2,
            wall_height: 2.0,
            seed: 11,
        }),
    };

    let first = run_arena(&config);
    let second = run_arena(&config);

    assert_eq!(first, second);
    assert!(!first.final_snapshot.actors.is_empty());
    assert!(first.event_log.events.len() >= config.tick_count as usize);
}

#[test]
fn arena_generated_ship_satisfies_map_contracts() {
    let ship = ArenaShipConfig {
        grid_cols: 2,
        grid_rows: 1,
        min_room_size: 4.0,
        max_room_size: 4.0,
        corridor_width: 1.0,
        wall_thickness: 0.2,
        wall_height: 2.0,
        seed: 11,
    };

    let data = generate_ship_data(&ship).expect("ship data generation should succeed");

    MapAssert::new("arena generated ship", &data.grid, &data.graph)
        .room_graph_connected()
        .passable_connected()
        .no_orphan_doors()
        .assert();
}

#[test]
fn arena_world_reset_is_leak_free() {
    let config = ArenaConfig {
        seed: 19,
        tick_count: 5,
        actors: vec![ArenaActorSpawn {
            translation: Vec3::new(0.0, 0.25, 0.0),
            velocity: Vec3::default(),
            radius: 0.1667,
            yaw: 0.0,
            angular_velocity: 0.0,
        }],
        ship: Some(ArenaShipConfig {
            grid_cols: 2,
            grid_rows: 1,
            min_room_size: 4.0,
            max_room_size: 4.0,
            corridor_width: 1.0,
            wall_thickness: 0.2,
            wall_height: 2.0,
            seed: 11,
        }),
    };

    let mut app = build_arena_app(&config);
    let expected_entities = count_gameplay_entities(&mut app);

    for _ in 0..5 {
        reset_arena_world(&mut app.world, &config);
        let actual_entities = count_gameplay_entities(&mut app);
        assert_eq!(
            expected_entities, actual_entities,
            "arena world reset leaked gameplay entities"
        );
    }
}

fn count_gameplay_entities(app: &mut App) -> usize {
    let mut count = 0;
    let mut query = app.world.query::<&spacetrash_v11::components::GameplayEntity>();
    for _ in query.iter(&app.world) {
        count += 1;
    }
    count
}
