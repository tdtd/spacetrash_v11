use std::collections::{HashMap, HashSet};

use spacetrash_v11::procgen::wfc::{
    run_wfc_with_settings, TileAdjacency, WfcBalanceProfile, WfcGrid, WfcSettings, WfcTileSet,
};

fn simple_tile_set() -> WfcTileSet {
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

    WfcTileSet::new(tiles, adjacency)
}

#[test]
fn weighted_profile_favors_high_weight_tile() {
    let mut counts = [0usize; 2];

    for seed in 0..100 {
        let mut grid = WfcGrid::new(1, 1, &simple_tile_set().tiles);
        let balance = WfcBalanceProfile {
            tile_weight_multipliers: HashMap::from([(1, 10.0), (0, 1.0)]),
        };
        let settings = WfcSettings { seed };

        let result = run_wfc_with_settings(&mut grid, &simple_tile_set(), &settings, &balance)
            .expect("WFC should collapse");

        let tile = result.tiles[0][0];
        counts[tile] += 1;
    }

    assert!(
        counts[1] > 70,
        "high weight tile should appear much more often, counts: {:?}",
        counts
    );
}

#[test]
fn same_seed_produces_identical_grid() {
    let tile_set = simple_tile_set();
    let seed = 42;

    let mut grid_a = WfcGrid::new(4, 4, &tile_set.tiles);
    let mut grid_b = WfcGrid::new(4, 4, &tile_set.tiles);

    let balance = WfcBalanceProfile::default();
    let settings = WfcSettings { seed };

    let result_a =
        run_wfc_with_settings(&mut grid_a, &tile_set, &settings, &balance).unwrap();
    let result_b =
        run_wfc_with_settings(&mut grid_b, &tile_set, &settings, &balance).unwrap();

    assert_eq!(result_a.tiles, result_b.tiles);
}

#[test]
fn zero_weight_is_rejected() {
    let tile_set = simple_tile_set();
    let mut grid = WfcGrid::new(1, 1, &tile_set.tiles);

    let balance = WfcBalanceProfile {
        tile_weight_multipliers: HashMap::from([(0, 0.0), (1, 0.0)]),
    };
    let settings = WfcSettings { seed: 1 };

    let err = run_wfc_with_settings(&mut grid, &tile_set, &settings, &balance)
        .expect_err("all zero weights must fail");

    assert!(err.contains("non-positive weight"));
}
