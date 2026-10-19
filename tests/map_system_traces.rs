use spacetrash_v11::procgen::scaffold::{RoomGraph, SystemFlags};
use spacetrash_v11::procgen::tile::TileType;
use spacetrash_v11::testing::map_assertions::MapAssert;

#[test]
fn connected_conduit_chain_satisfies_system_connectivity() {
    let graph = RoomGraph::new();

    let grid = vec![
        vec![TileType::Floor, TileType::Conduit, TileType::Floor],
        vec![TileType::Floor, TileType::Conduit, TileType::Floor],
        vec![TileType::Floor, TileType::Floor, TileType::Floor],
    ];

    MapAssert::new("connected conduit chain", &grid, &graph)
        .system_line_connectivity(SystemFlags::ELECTRICAL, 1)
        .assert();
}

#[test]
fn connected_pipe_chain_satisfies_system_connectivity() {
    let graph = RoomGraph::new();

    let grid = vec![
        vec![TileType::Pipe, TileType::Floor, TileType::Floor],
        vec![TileType::Pipe, TileType::Floor, TileType::Floor],
        vec![TileType::Floor, TileType::Floor, TileType::Floor],
    ];

    MapAssert::new("connected pipe chain", &grid, &graph)
        .system_line_connectivity(SystemFlags::FLUIDICS, 1)
        .assert();
}
