use std::collections::{HashMap, HashSet};

use spacetrash_v11::procgen::wfc::{run_wfc, Direction, TileAdjacency, WfcGrid, WfcTileSet};

fn simple_tile_set() -> (WfcTileSet, HashSet<usize>) {
    let tiles: HashSet<usize> = [0, 1].into_iter().collect();
    let mut adjacency = HashMap::new();
    for tile in &tiles {
        let mut adj = TileAdjacency::new();
        adj.north.insert(*tile);
        adj.south.insert(*tile);
        adj.east.insert(*tile);
        adj.west.insert(*tile);
        adjacency.insert(*tile, adj);
    }
    (WfcTileSet::new(tiles.clone(), adjacency), tiles)
}

#[test]
fn wfc_2x2_all_the_same_tile() {
    let (tile_set, all_tiles) = simple_tile_set();
    let mut grid = WfcGrid::new(2, 2, &all_tiles);
    let result = run_wfc(&mut grid, &tile_set).unwrap();
    let tiles = result.tiles;
    let first = tiles[0][0];
    assert!(tiles.iter().all(|row| row.iter().all(|&t| t == first)));
}

#[test]
fn wfc_respects_adjacency_constraints() {
    // Tiles: 0 and 1
    // Rules: each tile must be different from its orthogonal neighbors.
    let tiles: HashSet<usize> = [0, 1].into_iter().collect();
    let mut adjacency = HashMap::new();
    for &tile in &tiles {
        let mut adj = TileAdjacency::new();
        for dir in [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ] {
            let allowed = if tile == 0 { 1 } else { 0 };
            match dir {
                Direction::North => {
                    adj.north.insert(allowed);
                }
                Direction::South => {
                    adj.south.insert(allowed);
                }
                Direction::East => {
                    adj.east.insert(allowed);
                }
                Direction::West => {
                    adj.west.insert(allowed);
                }
            }
        }
        adjacency.insert(tile, adj);
    }

    let tile_set = WfcTileSet::new(tiles.clone(), adjacency);
    let mut grid = WfcGrid::new(3, 3, &tiles);
    let result = run_wfc(&mut grid, &tile_set).unwrap();
    let tiles = result.tiles;

    for y in 0..3 {
        for x in 0..3 {
            let current = tiles[y][x];
            if x + 1 < 3 {
                assert_ne!(
                    current, tiles[y][x + 1],
                    "East neighbor must be different at ({}, {})",
                    x, y
                );
            }
            if x > 0 {
                assert_ne!(current, tiles[y][x - 1]);
            }
            if y + 1 < 3 {
                assert_ne!(current, tiles[y + 1][x]);
            }
            if y > 0 {
                assert_ne!(current, tiles[y - 1][x]);
            }
        }
    }
}

#[test]
fn wfc_detects_contradiction() {
    // Both tiles have empty adjacency sets, so any propagation leads to contradiction.
    let tiles: HashSet<usize> = [0, 1].into_iter().collect();
    let mut adjacency = HashMap::new();
    for tile in &tiles {
        adjacency.insert(*tile, TileAdjacency::new());
    }

    let tile_set = WfcTileSet::new(tiles.clone(), adjacency);
    let mut grid = WfcGrid::new(2, 1, &tiles);
    let result = run_wfc(&mut grid, &tile_set);
    assert!(result.is_err());
}
