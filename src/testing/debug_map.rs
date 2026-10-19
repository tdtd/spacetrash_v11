use std::fmt::Write;

use crate::procgen::tile::{TileType, WallDirection};

/// Renders the full tile map using diagnostic glyphs.
///
/// This is intended for failed map tests, not for player-facing UI.
pub fn render_debug_map(grid: &[Vec<TileType>]) -> String {
    render_grid(grid, tile_debug_char)
}

/// Renders only physical passability.
///
/// `#` means blocked, `.` means passable, and `+` is used for door tiles.
pub fn render_passability_map(grid: &[Vec<TileType>]) -> String {
    render_grid(grid, tile_passability_char)
}

fn render_grid<F>(grid: &[Vec<TileType>], mut char_fn: F) -> String
where
    F: FnMut(&TileType) -> char,
{
    if grid.is_empty() || grid[0].is_empty() {
        return "(empty map)".to_string();
    }

    let height = grid.len();
    let width = grid[0].len();

    let mut out = String::new();

    // Column header.
    let mut header = String::from("   ");
    for x in 0..width {
        let _ = write!(header, "{}", x % 10);
    }
    out.push_str(&header);
    out.push('\n');

    for y in 0..height {
        let mut row = format!("{:>2} ", y);
        for x in 0..width {
            let tile = grid[y].get(x).copied().unwrap_or(TileType::Floor);
            row.push(char_fn(&tile));
        }
        out.push_str(&row);
        out.push('\n');
    }

    out
}

fn tile_debug_char(tile: &TileType) -> char {
    match tile {
        TileType::Floor => '.',
        TileType::Conduit => '*',
        TileType::Pipe => '~',
        TileType::Door(_) => '+',
        TileType::Wall(WallDirection::North) => 'N',
        TileType::Wall(WallDirection::South) => 'S',
        TileType::Wall(WallDirection::East) => 'E',
        TileType::Wall(WallDirection::West) => 'W',
        TileType::Wall(WallDirection::NorthEast) => '7',
        TileType::Wall(WallDirection::NorthWest) => '8',
        TileType::Wall(WallDirection::SouthEast) => '9',
        TileType::Wall(WallDirection::SouthWest) => '0',
        TileType::Void => ' ',
    }
}

fn tile_passability_char(tile: &TileType) -> char {
    match tile {
        TileType::Door(_) => '+',
        other if other.is_passable() => '.',
        TileType::Void => '#',
        _ => '#',
    }
}
