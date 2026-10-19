use serde::{Deserialize, Serialize};

/// Compass direction used by wall and door tile variants.
///
/// The eight variants exist for future diagonal interior walls, but room
/// boundaries currently use only the four cardinal directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WallDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

/// A tile in the generated floorplan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Wall(WallDirection),
    Door(WallDirection),
    /// Passable electrical conduit marker (used by electrical schematic layer).
    Conduit,
    /// Passable fluid pipe marker (used by fluidics schematic layer).
    Pipe,
    /// Non-room deadzone / outside the ship. Impassable, but not a wall.
    Void,
}

impl TileType {
    pub fn is_passable(&self) -> bool {
        matches!(
            self,
            TileType::Floor
                | TileType::Door(_)
                | TileType::Conduit
                | TileType::Pipe
        )
    }

    pub fn is_wall(&self) -> bool {
        matches!(self, TileType::Wall(_))
    }

    /// Returns the ASCII glyph used by the schematic map for this tile.
    ///
    /// Structural view uses the same glyph for mechanical floor and conduits/pipes.
    pub fn schematic_char(&self) -> char {
        match self {
            TileType::Floor => '.',
            TileType::Wall(_) => '#',
            TileType::Door(_) => '+',
            TileType::Conduit => '.',
            TileType::Pipe => '.',
            TileType::Void => ' ',
        }
    }
}
