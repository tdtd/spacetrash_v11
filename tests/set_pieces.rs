use bevy::prelude::*;

use spacetrash_v11::arena::{build_arena_app, ArenaConfig, ArenaShipConfig};
use spacetrash_v11::components::{
    AabbCollider, AiBehaviorState, BehaviorDirective, PhysicalSetPiece, SetPieceKind, Velocity,
};

fn app_with_ship(cols: usize, rows: usize, seed: u64) -> App {
    let config = ArenaConfig {
        seed,
        tick_count: 0,
        actors: vec![],
        ship: Some(ArenaShipConfig {
            grid_cols: cols,
            grid_rows: rows,
            min_room_size: 4.0,
            max_room_size: 4.0,
            corridor_width: 1.0,
            wall_thickness: 0.2,
            wall_height: 2.0,
            seed,
        }),
    };

    build_arena_app(&config)
}

#[test]
fn reactor_room_spawns_reactor_core_set_piece() {
    let mut app = app_with_ship(1, 1, 11);

    let mut query = app.world.query::<(&PhysicalSetPiece, &SetPieceKind)>();
    let mut reactor_core_count = 0;

    for (_, kind) in query.iter(&app.world) {
        if *kind == SetPieceKind::ReactorCore {
            reactor_core_count += 1;
        }
    }

    assert_eq!(
        reactor_core_count, 1,
        "single-room reactor ship should contain exactly one reactor core set piece"
    );
}

#[test]
fn storage_room_spawns_cargo_box_set_piece() {
    let mut app = app_with_ship(3, 3, 11);

    let mut query = app.world.query::<(&PhysicalSetPiece, &SetPieceKind)>();
    let mut cargo_box_count = 0;

    for (_, kind) in query.iter(&app.world) {
        if *kind == SetPieceKind::CargoBox {
            cargo_box_count += 1;
        }
    }

    assert!(
        cargo_box_count >= 1,
        "ship with storage rooms should contain at least one cargo box set piece"
    );
}

#[test]
fn set_piece_has_collider_but_no_agent_components() {
    let mut app = app_with_ship(3, 3, 11);

    let entities: Vec<Entity> = {
        let mut query = app.world.query::<(Entity, &PhysicalSetPiece)>();
        query.iter(&app.world).map(|(entity, _)| entity).collect()
    };

    assert!(!entities.is_empty(), "expected physical set pieces");

    for entity in entities {
        assert!(
            app.world.get::<AabbCollider>(entity).is_some(),
            "set piece must have an AabbCollider"
        );
        assert!(
            app.world.get::<Velocity>(entity).is_none(),
            "set piece must not have movement"
        );
        assert!(
            app.world.get::<BehaviorDirective>(entity).is_none(),
            "set piece must not have behavior"
        );
        assert!(
            app.world.get::<AiBehaviorState>(entity).is_none(),
            "set piece must not have an AI state"
        );
    }
}
