use spacetrash_v11::procgen::composite::{
    merge_instances, InstancePlacement, UmbilicalConnection,
};
use spacetrash_v11::procgen::scaffold::{
    EdgeKind, EdgePlacement, Rect, RoomEdge, RoomGraph, RoomNode, RoomType,
};
use spacetrash_v11::procgen::tile::TileType;
use spacetrash_v11::testing::map_assertions::MapAssert;

fn ship_airlock_graph() -> RoomGraph {
    let mut graph = RoomGraph::new();
    graph.add_node(RoomNode::new(
        0,
        RoomType::Airlock,
        Rect::new(0.0, 0.0, 1.0, 1.0),
    ));
    graph
}

fn ship_airlock_grid() -> Vec<Vec<TileType>> {
    vec![vec![TileType::Floor]]
}

fn station_graph() -> RoomGraph {
    let mut graph = RoomGraph::new();

    // Central storage hub.
    graph.add_node(RoomNode::new(
        0,
        RoomType::Storage,
        Rect::new(2.0, 2.0, 1.0, 1.0),
    ));

    // North airlock.
    graph.add_node(RoomNode::new(
        1,
        RoomType::Airlock,
        Rect::new(2.0, 4.0, 1.0, 1.0),
    ));

    // East airlock.
    graph.add_node(RoomNode::new(
        2,
        RoomType::Airlock,
        Rect::new(4.0, 2.0, 1.0, 1.0),
    ));

    // South airlock.
    graph.add_node(RoomNode::new(
        3,
        RoomType::Airlock,
        Rect::new(2.0, 0.0, 1.0, 1.0),
    ));

    // West airlock.
    graph.add_node(RoomNode::new(
        4,
        RoomType::Airlock,
        Rect::new(0.0, 2.0, 1.0, 1.0),
    ));

    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 1,
        location: EdgePlacement::North,
    });
    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 2,
        location: EdgePlacement::East,
    });
    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 3,
        location: EdgePlacement::South,
    });
    graph.add_edge(RoomEdge {
        kind: EdgeKind::Door,
        from: 0,
        to: 4,
        location: EdgePlacement::West,
    });

    graph
}

fn station_grid() -> Vec<Vec<TileType>> {
    let mut grid = vec![vec![TileType::Void; 5]; 5];

    // Interior 3x3 block.
    for y in 1..=3 {
        for x in 1..=3 {
            grid[y][x] = TileType::Floor;
        }
    }

    // Four airlock arms.
    grid[4][2] = TileType::Floor; // north
    grid[2][4] = TileType::Floor; // east
    grid[0][2] = TileType::Floor; // south
    grid[2][0] = TileType::Floor; // west

    grid
}

#[test]
fn four_sided_station_merges_into_single_connected_level() {
    // Station is instance 0.
    let station_placement = InstancePlacement {
        graph: station_graph(),
        grid: station_grid(),
        offset_x: 2,
        offset_y: 2,
    };

    // Four ships are instances 1..=4.
    let north_ship = InstancePlacement {
        graph: ship_airlock_graph(),
        grid: ship_airlock_grid(),
        offset_x: 4,
        offset_y: 7,
    };

    let east_ship = InstancePlacement {
        graph: ship_airlock_graph(),
        grid: ship_airlock_grid(),
        offset_x: 7,
        offset_y: 4,
    };

    let south_ship = InstancePlacement {
        graph: ship_airlock_graph(),
        grid: ship_airlock_grid(),
        offset_x: 4,
        offset_y: 1,
    };

    let west_ship = InstancePlacement {
        graph: ship_airlock_graph(),
        grid: ship_airlock_grid(),
        offset_x: 1,
        offset_y: 4,
    };

    let instances = vec![
        station_placement,
        north_ship,
        east_ship,
        south_ship,
        west_ship,
    ];

    let umbilical_connections = vec![
        UmbilicalConnection {
            from_instance: 1,
            from_room: 0,
            to_instance: 0,
            to_room: 1,
        },
        UmbilicalConnection {
            from_instance: 2,
            from_room: 0,
            to_instance: 0,
            to_room: 2,
        },
        UmbilicalConnection {
            from_instance: 3,
            from_room: 0,
            to_instance: 0,
            to_room: 3,
        },
        UmbilicalConnection {
            from_instance: 4,
            from_room: 0,
            to_instance: 0,
            to_room: 4,
        },
    ];

    let composite = merge_instances(instances, umbilical_connections)
        .expect("composite four-sided station merge should succeed");

    MapAssert::new("four-sided station composite", &composite.grid, &composite.graph)
        .room_graph_connected()
        .passable_connected()
        .assert();
}
