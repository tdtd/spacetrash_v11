use spacetrash_v11::procgen::graph_enhanced_wfc::{
    graph_enhanced_collapse, GraphWfcBalanceProfile, GraphWfcConfig,
};
use spacetrash_v11::procgen::scaffold::{Rect, RoomGraph, RoomNode, RoomType};
use spacetrash_v11::procgen::tile::TileType;

fn single_room_grid(room_type: RoomType) -> Vec<Vec<TileType>> {
    let mut graph = RoomGraph::new();
    graph.add_node(RoomNode::new(0, room_type, Rect::new(0.0, 0.0, 5.0, 5.0)));

    let config = GraphWfcConfig {
        width: 5,
        height: 5,
        seed: 1,
    };

    graph_enhanced_collapse(&config, &graph, &GraphWfcBalanceProfile::default())
        .expect("single-room WFC collapse should succeed")
}

fn count_tile(grid: &[Vec<TileType>], expected: TileType) -> usize {
    grid.iter()
        .flatten()
        .filter(|&&tile| tile == expected)
        .count()
}

#[test]
fn reactor_room_contains_electrical_terminal() {
    let grid = single_room_grid(RoomType::Reactor);

    assert!(
        count_tile(&grid, TileType::Conduit) >= 1,
        "reactor room should contain at least one electrical conduit terminal"
    );
}

#[test]
fn engineering_room_contains_electrical_terminal() {
    let grid = single_room_grid(RoomType::Engineering);

    assert!(
        count_tile(&grid, TileType::Conduit) >= 1,
        "engineering room should contain at least one electrical conduit terminal"
    );
}

#[test]
fn hydroponics_room_contains_fluidics_terminal() {
    let grid = single_room_grid(RoomType::Hydroponics);

    assert!(
        count_tile(&grid, TileType::Pipe) >= 1,
        "hydroponics room should contain at least one fluid pipe terminal"
    );
}

#[test]
fn storage_room_has_no_system_terminal() {
    let grid = single_room_grid(RoomType::Storage);

    assert_eq!(
        count_tile(&grid, TileType::Conduit),
        0,
        "storage room must not contain electrical conduit terminals"
    );
    assert_eq!(
        count_tile(&grid, TileType::Pipe),
        0,
        "storage room must not contain fluid pipe terminals"
    );
}
