use bevy::prelude::*;

use crate::resources::ship_template::{DoorOverride, DockingPortDef, ManualRoom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A procedural generation algorithm ID.
pub type ProcGenId = String;

/// Context passed to a procedural generation algorithm.
///
/// This can be expanded later to include the target ship/level template,
/// resource limits, and other world parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcGenContext {
    pub seed: u64,
    pub params: serde_json::Value,
}

/// Output of a ship-generation algorithm.
///
/// The fields are intentionally compatible with `ShipTemplate` manual
/// overrides so the result can later be transformed into a full template or
/// used directly by the ship generation system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShipGenOutput {
    pub rooms: Vec<ManualRoom>,
    pub door_overrides: Vec<DoorOverride>,
    pub docking_ports: Vec<DockingPortDef>,
}

/// Trait for procedural generation algorithms that produce ship layouts.
///
/// Implementations must be thread-safe (`Send + Sync`) because they may be
/// stored in a global registry and called from multiple systems later.
pub trait ShipGenerationAlgorithm: Send + Sync {
    fn generate(&self, ctx: &ProcGenContext) -> ShipGenOutput;
}

/// Global registry mapping algorithm IDs to concrete algorithm implementations.
///
/// This is a Bevy `Resource` so it can be inserted at startup and extended by
/// future code, including loaded mods, scripts, or WASM modules.
#[derive(Resource, Default)]
pub struct ProcGenRegistry {
    algorithms: HashMap<ProcGenId, Box<dyn ShipGenerationAlgorithm>>,
}

impl ProcGenRegistry {
    /// Registers a ship-generation algorithm under the given ID.
    pub fn register(&mut self, id: ProcGenId, algorithm: Box<dyn ShipGenerationAlgorithm>) {
        self.algorithms.insert(id, algorithm);
    }

    /// Returns a reference to a registered algorithm, if present.
    pub fn get(&self, id: &str) -> Option<&dyn ShipGenerationAlgorithm> {
        self.algorithms.get(id).map(|b| b.as_ref())
    }
}
