/// Returns a human-readable name for the internal WFC tile IDs used by
/// `graph_enhanced_wfc`.
///
/// Tests should use this instead of printing raw numeric tile IDs.
pub fn wfc_tile_name(tile_id: usize) -> String {
    match tile_id {
        0 => "Floor".to_string(),
        1 => "Conduit".to_string(),
        2 => "Pipe".to_string(),
        3 => "WallNorth".to_string(),
        4 => "WallSouth".to_string(),
        5 => "WallEast".to_string(),
        6 => "WallWest".to_string(),
        7 => "WallNorthEast".to_string(),
        8 => "WallNorthWest".to_string(),
        9 => "WallSouthEast".to_string(),
        10 => "WallSouthWest".to_string(),
        11 => "DoorNorth".to_string(),
        12 => "DoorSouth".to_string(),
        13 => "DoorEast".to_string(),
        14 => "DoorWest".to_string(),
        15 => "Void".to_string(),
        other => format!("UnknownTile({other})"),
    }
}
