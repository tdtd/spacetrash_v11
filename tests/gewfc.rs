use spacetrash_v11::procgen::graph_enhanced_wfc::{
    graph_enhanced_collapse, GraphWfcBalanceProfile, GraphWfcConfig,
};
use spacetrash_v11::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode, RoomType,
};
use spacetrash_v11::procgen::tile::{TileType, WallDirection};

#[test]
fn gewfc_preserves_door_between_two_rooms() {
    let mut graph = RoomGraph::new();

    let room_a = RoomNode::new(0, RoomType::Storage, Rect::new(0.0, 0.0, 4.0, 4.0));
    let room_b = RoomNode::new(1, RoomType::Engineering, Rect::new(4.0, 0.0, 4.0, 4.0));

    graph.add_node(room_a);
    graph.add_node(room_b);

    assert!(graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 1,
        location: EdgePlacement::East,
    }));

    let config = GraphWfcConfig {
        width: 8,
        height: 4,
        seed: 1,
    };

    let balance = GraphWfcBalanceProfile::default();
    let result = graph_enhanced_collapse(&config, &graph, &balance);

    assert!(
        result.is_ok(),
        "Graph Enhanced WFC should produce a layout for a simple two-room graph, got {:?}",
        result
    );

    let grid = result.unwrap();
    assert_eq!(grid.len(), 4);
    assert_eq!(grid[0].len(), 8);
    assert!(grid.iter().flatten().any(|t| matches!(t, TileType::Door(_))));
}

#[test]
fn single_room_generates_all_eight_wall_directions() {
    let mut graph = RoomGraph::new();
    graph.add_node(RoomNode::new(0, RoomType::Storage, Rect::new(0.0, 0.0, 4.0, 4.0)));

    let config = GraphWfcConfig {
        width: 4,
        height: 4,
        seed: 1,
    };

    let balance = GraphWfcBalanceProfile::default();
    let grid = graph_enhanced_collapse(&config, &graph, &balance)
        .expect("single-room collapse should succeed");

    assert_eq!(grid[0][0], TileType::Wall(WallDirection::SouthWest));
    assert_eq!(grid[0][3], TileType::Wall(WallDirection::SouthEast));
    assert_eq!(grid[3][0], TileType::Wall(WallDirection::NorthWest));
    assert_eq!(grid[3][3], TileType::Wall(WallDirection::NorthEast));

    assert_eq!(grid[1][0], TileType::Wall(WallDirection::West));
    assert_eq!(grid[1][3], TileType::Wall(WallDirection::East));
    assert_eq!(grid[0][1], TileType::Wall(WallDirection::South));
    assert_eq!(grid[3][1], TileType::Wall(WallDirection::North));

    let mut interior_tiles: Vec<TileType> = Vec::new();
    for y in 1..3 {
        for x in 1..3 {
            interior_tiles.push(grid[y][x]);
        }
    }

    assert!(
        interior_tiles
            .iter()
            .all(|t| *t != TileType::Wall(WallDirection::North))
    );
}
