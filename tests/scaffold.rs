use spacetrash_v11::procgen::scaffold::*;

#[test]
fn system_flags_for_room_types() {
    assert_eq!(RoomType::Storage.system_flags(), SystemFlags::empty());
    assert_eq!(RoomType::Engineering.system_flags(), SystemFlags::ELECTRICAL);
    assert_eq!(
        RoomType::Reactor.system_flags(),
        SystemFlags::ELECTRICAL | SystemFlags::THERMAL
    );
    assert_eq!(RoomType::LifeSupport.system_flags(), SystemFlags::ATMOSPHERICS);
    assert_eq!(
        RoomType::Hydroponics.system_flags(),
        SystemFlags::FLUIDICS | SystemFlags::ATMOSPHERICS
    );
    assert_eq!(RoomType::ComputerCore.system_flags(), SystemFlags::COMPUTER);
    assert_eq!(
        RoomType::Bridge.system_flags(),
        SystemFlags::COMPUTER | SystemFlags::ELECTRICAL
    );
}

#[test]
fn room_graph_add_and_neighbors() {
    let mut graph = RoomGraph::new();
    let rect = Rect::new(0.0, 0.0, 1.0, 1.0);

    graph.add_node(RoomNode::new(1, RoomType::Storage, rect));
    graph.add_node(RoomNode::new(2, RoomType::Engineering, rect));

    let added = graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 1,
        to: 2,
        location: EdgePlacement::North,
    });

    assert!(added);
    let neighbors = graph.neighbors(1);
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0], 2);
}

#[test]
fn graph_is_connected() {
    let mut graph = RoomGraph::new();
    let rect = Rect::new(0.0, 0.0, 1.0, 1.0);

    for id in 1..=4 {
        graph.add_node(RoomNode::new(id, RoomType::Storage, rect));
    }

    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 1,
        to: 2,
        location: EdgePlacement::East,
    });
    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 2,
        to: 3,
        location: EdgePlacement::North,
    });
    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 3,
        to: 4,
        location: EdgePlacement::West,
    });

    assert!(graph.is_connected());
}

#[test]
fn graph_is_disconnected_when_missing_edge() {
    let mut graph = RoomGraph::new();
    let rect = Rect::new(0.0, 0.0, 1.0, 1.0);

    graph.add_node(RoomNode::new(1, RoomType::Storage, rect));
    graph.add_node(RoomNode::new(2, RoomType::Storage, rect));

    assert!(!graph.is_connected());
}
