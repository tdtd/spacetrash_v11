use bevy::prelude::*;

use crate::components::{HeatAccumulator, SetPieceKind, ThermalEmitter};
use crate::procgen::scaffold::RoomType;

pub fn set_piece_kind_for_room(room_type: RoomType) -> Option<SetPieceKind> {
    match room_type {
        RoomType::Storage => Some(SetPieceKind::CargoBox),
        RoomType::Engineering => Some(SetPieceKind::PowerDistributor),
        RoomType::Reactor => Some(SetPieceKind::ReactorCore),
        RoomType::LifeSupport => Some(SetPieceKind::AirScrubber),
        RoomType::Hydroponics => Some(SetPieceKind::HydroponicBay),
        RoomType::ComputerCore => Some(SetPieceKind::ServerRack),
        RoomType::Bridge => Some(SetPieceKind::CommandConsole),
        RoomType::Airlock => Some(SetPieceKind::AirlockMechanism),
    }
}

pub fn set_piece_half_extents(kind: SetPieceKind) -> Vec3 {
    match kind {
        SetPieceKind::CargoBox => Vec3::new(0.35, 0.35, 0.35),
        SetPieceKind::ReactorCore => Vec3::new(0.45, 0.45, 0.45),
        SetPieceKind::PowerDistributor => Vec3::new(0.40, 0.40, 0.40),
        SetPieceKind::AirScrubber => Vec3::new(0.40, 0.60, 0.40),
        SetPieceKind::HydroponicBay => Vec3::new(0.80, 0.30, 0.50),
        SetPieceKind::ServerRack => Vec3::new(0.30, 0.90, 0.60),
        SetPieceKind::CommandConsole => Vec3::new(0.50, 0.50, 0.40),
        SetPieceKind::AirlockMechanism => Vec3::new(0.35, 0.35, 0.35),
    }
}

pub fn set_piece_visual_size(kind: SetPieceKind) -> Vec3 {
    set_piece_half_extents(kind) * 2.0
}

/// Simple heat transfer from thermal emitters to accumulators in range.
pub fn set_piece_thermal_system(
    time: Res<Time>,
    emitters: Query<(&ThermalEmitter, &Transform)>,
    mut accumulators: Query<(&Transform, &mut HeatAccumulator)>,
) {
    let dt = time.delta_seconds();

    for (emitter, emitter_transform) in emitters.iter() {
        for (accumulator_transform, mut accumulator) in accumulators.iter_mut() {
            let distance = emitter_transform
                .translation
                .distance(accumulator_transform.translation);

            if distance <= emitter.range {
                accumulator.heat += emitter.intensity * dt;
            }
        }
    }
}
