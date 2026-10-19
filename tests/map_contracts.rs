use spacetrash_v11::procgen::graph_enhanced_wfc::{
    graph_enhanced_collapse, GraphWfcBalanceProfile, GraphWfcConfig,
};
use spacetrash_v11::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode, RoomType,
};
use spacetrash_v11::testing::map_assertions::MapAssert;

#[test]
fn basic_two_room_map_satisfies_initial_contract() {
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
        seed: 1,
    };

    let balance = GraphWfcBalanceProfile::default();
    let grid = graph_enhanced_collapse(&config, &graph, &balance)
        .expect("two-room collapse should succeed");

    MapAssert::new("two-room map contract", &grid, &graph)
        .size(8, 4)
        .has_room_type(RoomType::Storage)
        .has_room_type(RoomType::Engineering)
        .room_count_between(2..=2)
        .room_graph_connected()
        .passable_connected()
        .no_orphan_doors()
        .assert();
}
