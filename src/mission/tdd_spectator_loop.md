# Headless / Spectator TDD Loop

## Goal

Use one deterministic simulation core in both:

- headless `cargo test`
- headful developer spectator mode

The LLM/developer loop is:

1. Write or modify a map/simulation contract test.
2. Run:

```bash
cargo test --all-targets -- --nocapture
```

3. If a map assertion fails, read the complete map report:

   - debug map
   - passability map
   - invariant summary

4. Refine the relevant generator/WFC/behavior code.
5. Repeat.

Avoid launching the game to guess. Use spectator only to inspect a
successful/failing configuration visually after the tests reproduce it.

## Single Simulation Core

- `ArenaConfig`
- `ArenaActorSpawn`
- `ArenaShipConfig`
- `generate_ship_data`
- `build_arena_world`
- `run_arena`
- `ArenaRunResult`
- `ArenaEventLog`

The game spectator is a headful projection of a headless `run_arena`.
It does not implement separate physics, AI, or generation.

## Headless Tests

Primary contracts:

- `tests/wfc_regression_matrix.rs`
- `tests/map_contracts.rs`
- `tests/map_diversity.rs`
- `tests/map_system_traces.rs`
- `tests/simulation_regression.rs`
- `tests/arena.rs`

## Spectator

Enter gameplay, pause, choose `Enter Spectator`.

Spectator runs a built-in `ArenaConfig` through `run_arena`, then plays
the recorded snapshots. It is a developer inspection tool, not normal
player gameplay.
