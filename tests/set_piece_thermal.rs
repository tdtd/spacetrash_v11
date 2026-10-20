use bevy::prelude::*;
use spacetrash_v11::components::{HeatAccumulator, ThermalEmitter};
use spacetrash_v11::systems::set_pieces::set_piece_thermal_system;
use spacetrash_v11::testing::{run_n_ticks, test_app};

#[test]
fn thermal_emitter_heats_accumulator_within_range() {
    let mut app = test_app();

    app.world.spawn((
        ThermalEmitter {
            intensity: 5.0,
            range: 3.0,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    app.world.spawn((
        HeatAccumulator::default(),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    app.add_systems(Update, set_piece_thermal_system);

    run_n_ticks(&mut app, 60);

    let mut query = app.world.query::<&HeatAccumulator>();
    for accumulator in query.iter(&app.world) {
        assert!(
            (accumulator.heat - 5.0).abs() < 0.001,
            "expected heat 5.0, got {}",
            accumulator.heat
        );
    }
}

#[test]
fn thermal_emitter_does_not_heat_accumulator_outside_range() {
    let mut app = test_app();

    app.world.spawn((
        ThermalEmitter {
            intensity: 5.0,
            range: 1.0,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    app.world.spawn((
        HeatAccumulator::default(),
        Transform::from_xyz(0.0, 0.0, 2.0),
    ));

    app.add_systems(Update, set_piece_thermal_system);

    run_n_ticks(&mut app, 60);

    let mut query = app.world.query::<&HeatAccumulator>();
    for accumulator in query.iter(&app.world) {
        assert_eq!(accumulator.heat, 0.0);
    }
}
