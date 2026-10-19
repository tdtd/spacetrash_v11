use spacetrash_v11::procgen::tile::{TileType, WallDirection};

#[test]
fn tile_type_passability() {
    assert!(TileType::Floor.is_passable());
    assert!(TileType::Door(WallDirection::North).is_passable());
    assert!(!TileType::Wall(WallDirection::East).is_passable());
}

#[test]
fn tile_type_wall_detection() {
    assert!(TileType::Wall(WallDirection::South).is_wall());
    assert!(!TileType::Floor.is_wall());
}

#[test]
fn tile_type_schematic_characters() {
    assert_eq!(TileType::Floor.schematic_char(), '.');
    assert_eq!(TileType::Wall(WallDirection::North).schematic_char(), '#');
    assert_eq!(TileType::Door(WallDirection::North).schematic_char(), '+');
}
