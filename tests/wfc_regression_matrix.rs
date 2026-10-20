use spacetrash_v11::arena::{generate_ship_data, ArenaShipConfig};
use spacetrash_v11::testing::map_assertions::MapAssert;

/// Builds a deterministic arena ship and asserts the map contracts that
/// matter for level quality.
///
/// This is the primary LLM feedback loop test:
/// - Run `cargo test wfc_regression_matrix -- --nocapture`
/// - Any failure prints a full debug map and passability map.
/// - Refine the WFC/generator until all seeds pass.
#[test]
fn generated_arena_ships_satisfy_map_contracts_across_seeds() {
    for seed in 1..=25u64 {
        let ship = ArenaShipConfig {
            grid_cols: 2,
            grid_rows: 2,
            min_room_size: 4.0,
            max_room_size: 4.0,
            corridor_width: 1.0,
            wall_thickness: 0.2,
            wall_height: 2.0,
            seed,
        };

        let data = generate_ship_data(&ship)
            .unwrap_or_else(|e| panic!("seed {seed} failed generation: {e}"));

        MapAssert::new(format!("arena ship seed {seed}"), &data.grid, &data.graph)
            .room_graph_connected()
            .passable_connected()
            .no_orphan_doors()
            .assert();
    }
}
