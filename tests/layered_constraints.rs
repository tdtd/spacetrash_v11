use spacetrash_v11::procgen::layer::{
    derive_graph_constraints, LayerKind, LayerTile, PathRequirement,
};
use spacetrash_v11::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode, RoomType,
};

#[test]
fn reactor_and_bridge_require_electrical_path() {
    let mut graph = RoomGraph::new();

    graph.add_node(RoomNode::new(
        0,
        RoomType::Reactor,
        Rect::new(0.0, 0.0, 4.0, 4.0),
    ));
    graph.add_node(RoomNode::new(
        1,
        RoomType::Bridge,
        Rect::new(5.0, 0.0, 4.0, 4.0),
    ));

    assert!(graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 1,
        location: EdgePlacement::East,
    }));

    let constraints = derive_graph_constraints(&graph);

    assert!(
        constraints.path_requirements.iter().any(|p| {
            p.layer == LayerKind::Electrical
                && p.from.y == p.to.y
                && p.from.x != p.to.x
        }),
        "expected an electrical path between Reactor and Bridge"
    );
}

#[test]
fn hydroponics_room_requires_fluidic_and_atmospheric_terminals() {
    let mut graph = RoomGraph::new();

    graph.add_node(RoomNode::new(
        0,
        RoomType::Hydroponics,
        Rect::new(0.0, 0.0, 3.0, 3.0),
    ));

    let constraints = derive_graph_constraints(&graph);

    let has_fluidic_terminal = constraints.cell_requirements.iter().any(|req| {
        req.layer == LayerKind::Fluidic && req.required_tile == LayerTile::Terminal
    });
    let has_atmospheric_terminal = constraints.cell_requirements.iter().any(|req| {
        req.layer == LayerKind::Atmospheric && req.required_tile == LayerTile::Terminal
    });

    assert!(has_fluidic_terminal);
    assert!(has_atmospheric_terminal);
}

#[test]
fn storage_rooms_do_not_create_system_paths() {
    let mut graph = RoomGraph::new();

    graph.add_node(RoomNode::new(
        0,
        RoomType::Storage,
        Rect::new(0.0, 0.0, 2.0, 2.0),
    ));
    graph.add_node(RoomNode::new(
        1,
        RoomType::Storage,
        Rect::new(3.0, 0.0, 2.0, 2.0),
    ));

    assert!(graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 1,
        location: EdgePlacement::East,
    }));

    let constraints = derive_graph_constraints(&graph);

    assert!(
        constraints
            .path_requirements
            .iter()
            .all(|p| p.layer != LayerKind::Electrical),
        "storage rooms have no electrical system, so no electrical path should be required"
    );
}
