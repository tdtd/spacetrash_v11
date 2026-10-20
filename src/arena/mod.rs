pub mod behavior;
pub mod config;
pub mod event;
pub mod runner;
pub mod session;
pub mod upgrades;
pub mod world_builder;

pub use behavior::{arbitrate_behavior, BehaviorSignals, InstalledBehaviorSet};
pub use config::{ArenaActorSpawn, ArenaConfig, ArenaShipConfig};
pub use event::{ActorState, ArenaEvent, ArenaEventLog, ArenaRunResult, ArenaSnapshot};
pub use runner::{build_arena_app, run_arena};
pub use session::{SimulationSession, SimulationSpeed, SimulationState};
pub use upgrades::{
    build_actor, ActorBuildSpec, ChassisDefinition, MovementProfile, SensorKind,
    UpgradeCategory, UpgradeDefinition, UpgradeRegistry, WeaponKind, BehaviorKind,
};
pub use world_builder::{build_arena_world, generate_ship_data, reset_arena_world, ShipData};
