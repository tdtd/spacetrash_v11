use std::collections::HashSet;

use spacetrash_v11::procgen::graph_enhanced_wfc::{
    graph_enhanced_collapse, GraphWfcBalanceProfile, GraphWfcConfig,
};
use spacetrash_v11::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode, RoomType,
};
use spacetrash_v11::procgen::tile::TileType;

#[test]
fn seeded_maps_have_reasonable_signature_variety() {
    let mut signatures = HashSet::new();

    for seed in 1..=10 {
        let grid = generate_two_room_map(seed);
        signatures.insert(map_signature(&grid));
    }

    assert!(
        signatures.len() >= 2,
        "generation is too samey; only {} distinct signatures",
        signatures.len()
    );
}

fn generate_two_room_map(seed: u64) -> Vec<Vec<TileType>> {
    let mut graph = RoomGraph::new();

    graph.add_node(RoomNode::new(
        0,
        RoomType::Storage,
        Rect::new(0.0, 0.0, 4.0, 4.0),
    ));
    graph.add_node(RoomNode::new(
        1,
        RoomType::Engineering,
        Rect::new(4.0, 0.0, 4.0, 4.0),
    ));

    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 1,
        location: EdgePlacement::East,
    });

    let config = GraphWfcConfig {
        width: 8,
        height: 4,
        seed,
    };

    let balance = GraphWfcBalanceProfile::default();
    graph_enhanced_collapse(&config, &graph, &balance)
        .expect("two-room collapse should succeed")
}

fn map_signature(grid: &[Vec<TileType>]) -> String {
    let mut floor = 0;
    let mut wall = 0;
    let mut door = 0;
    let mut conduit = 0;
    let mut pipe = 0;
    let mut void = 0;

    for tile in grid.iter().flatten() {
        match tile {
            TileType::Floor => floor += 1,
            TileType::Wall(_) => wall += 1,
            TileType::Door(_) => door += 1,
            TileType::Conduit => conduit += 1,
            TileType::Pipe => pipe += 1,
            TileType::Void => void += 1,
        }
    }

    format!("f{floor}_w{wall}_d{door}_c{conduit}_p{pipe}_v{void}")
}
