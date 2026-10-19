use std::fmt;

use crate::procgen::graph_enhanced_wfc::{
    graph_enhanced_collapse, GraphWfcBalanceProfile, GraphWfcConfig, GraphWfcError,
};
use crate::procgen::scaffold::RoomGraph;
use crate::procgen::tile::TileType;

/// High-level flavor for a generated ship.
///
/// Themes do not replace the WFC engine. They describe what a generated
/// ship should feel like so the same solver can produce industrial,
/// derelict, military, or later alien/research ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipTheme {
    Industrial,
    Derelict,
    Military,
}

/// Visual/structural wall style associated with a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallStyle {
    Solid,
    Segmented,
    Hull,
}

impl fmt::Display for WallStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WallStyle::Solid => write!(f, "solid"),
            WallStyle::Segmented => write!(f, "segmented"),
            WallStyle::Hull => write!(f, "hull"),
        }
    }
}

/// Theme-specific rule pack.
///
/// The stable WFC core should only read this data. Adding a new ship type
/// should usually mean adding a new rule pack, not touching the solver.
#[derive(Debug, Clone, PartialEq)]
pub struct WfcRulePack {
    pub name: String,
    pub theme: ShipTheme,
    pub wall_style: WallStyle,
    pub allows_concave_corners: bool,
    pub door_density: f32,
    pub deadzone_density: f32,
    pub wall_thickness_hint: f32,
    pub diagonal_wall_ratio: f32,
    pub system_tile_ratio: f32,
}

impl WfcRulePack {
    pub fn industrial() -> Self {
        Self {
            name: "industrial".to_string(),
            theme: ShipTheme::Industrial,
            wall_style: WallStyle::Solid,
            allows_concave_corners: false,
            door_density: 0.8,
            deadzone_density: 0.2,
            wall_thickness_hint: 0.2,
            diagonal_wall_ratio: 0.0,
            system_tile_ratio: 0.3,
        }
    }

    pub fn derelict() -> Self {
        Self {
            name: "derelict_hulk".to_string(),
            theme: ShipTheme::Derelict,
            wall_style: WallStyle::Hull,
            allows_concave_corners: true,
            door_density: 0.4,
            deadzone_density: 0.6,
            wall_thickness_hint: 0.25,
            diagonal_wall_ratio: 0.25,
            system_tile_ratio: 0.12,
        }
    }

    pub fn military() -> Self {
        Self {
            name: "military".to_string(),
            theme: ShipTheme::Military,
            wall_style: WallStyle::Segmented,
            allows_concave_corners: false,
            door_density: 0.6,
            deadzone_density: 0.2,
            wall_thickness_hint: 0.3,
            diagonal_wall_ratio: 0.05,
            system_tile_ratio: 0.35,
        }
    }
}

/// Stable generation configuration for the first semantic WFC pass.
#[derive(Debug, Clone)]
pub struct SemanticShipConfig {
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub rule_pack: WfcRulePack,
}

/// Generate a ship tile map using a skeleton graph and theme rule pack.
///
/// This first version intentionally delegates to the existing
/// `graph_enhanced_collapse` implementation. The theme and rule pack are
/// stored for the next phase, where they will control semantic domains,
/// weights, adjacency rules, and validation before the final collapse.
///
/// Keeping this wrapper now gives us a single call site that will become
/// the real semantic WFC pipeline without breaking current spawning code
/// when the internal implementation changes.
pub fn generate_ship_with_theme(
    config: &SemanticShipConfig,
    graph: &RoomGraph,
) -> Result<Vec<Vec<TileType>>, SemanticWfcError> {
    let wfc_config = GraphWfcConfig {
        width: config.width,
        height: config.height,
        seed: config.seed,
    };

    let balance = GraphWfcBalanceProfile::default();

    graph_enhanced_collapse(&wfc_config, graph, &balance).map_err(SemanticWfcError::Wfc)
}

#[derive(Debug)]
pub enum SemanticWfcError {
    Wfc(GraphWfcError),
}

impl fmt::Display for SemanticWfcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticWfcError::Wfc(e) => write!(f, "{:?}", e),
        }
    }
}
