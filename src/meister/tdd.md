# TDD-Driven Map / Procgen Testing

This document defines a test-first workflow for procedural generation and
map work. The goal is to let a human developer make strong assertions about
generated maps while giving an LLM or automated coding agent rich, actionable
debug output whenever a test fails.

The process must remain useful even if the WFC/backend solver is replaced.
Tests describe desired map behavior, not implementation details.

---

## Guiding Principles

1. **Map-level assertions, not solver internals**

   Tests should assert properties of the generated map, room graph, system
   graph, and visible tile map. They should not depend on whether a graph or
   local WFC algorithm produced the result.

2. **Properties over exact golden maps**

   Avoid locking a whole seed to a single exact ASCII layout. That reduces
   variety and makes generator changes too brittle.

   Prefer:

   - required room types exist
   - room graph is connected
   - no orphan doors
   - passable areas are connected
   - system density is within allowed range
   - no forbidden local patterns

3. **Every failure prints a complete map autopsy**

   A failed test should contain:

   - seed and config
   - rendered debug map
   - passability map
   - room graph dump
   - system graph dump
   - WFC contradiction/trace dump
   - invariant summary

   This output should be sufficient for an LLM to reproduce and reason about
   the bug without running the game client.

4. **Every visible WFC tile ID has a human-readable name**

   Raw numeric IDs are forbidden in diagnostics. The test helper must map
   internal tile IDs to names such as:

   ```text
   Floor
   Conduit
   Pipe
   WallNorth
   WallSouth
   WallEast
   WallWest
   WallNorthEast
   WallNorthWest
   WallSouthEast
   WallSouthWest
   DoorNorth
   DoorSouth
   DoorEast
   DoorWest
   Void
   ```

5. **Deterministic seeds are used for reproduction**

   Fixed-seed tests are allowed for reproduction and smoke tests. Contract
   tests may use a configured seed or a small set of seeds.

6. **Diversity is tested separately**

   Map variety is tested with signature distributions across many seeds, not
   by requiring many completely different fixed maps.

---

## Proposed Test Helper API

Add a map assertion entry point similar to this:

```rust
MapAssert::new(&grid, &graph, &config)
    .named("industrial seed 12345")

    // Structural
    .size(width, height)
    .no_orphan_floor()
    .passable_connected()
    .room_graph_connected()
    .no_orphan_doors()

    // Room type / progression
    .has_room_type(RoomType::Reactor)
    .has_room_type(RoomType::Bridge)
    .has_room_type(RoomType::LifeSupport)
    .room_count_between(4..=12)

    // System properties
    .room_has_system(RoomType::Reactor, SystemFlags::THERMAL)
    .system_density(SystemFlags::ELECTRICAL, 0.05..=0.35)

    // Local patterns
    .forbidden_2x2([
        [TileType::Floor, TileType::Wall(WallDirection::North)],
        [TileType::Wall(WallDirection::South), TileType::Wall(WallDirection::East)],
    ])

    .assert();
```

Each assertion method returns `self` so chaining is possible.

The final `assert()` performs all checks, collects every failure, and panics
with one complete report instead of stopping after the first failure.

---

## Recommended Module Layout

```text
src/testing/                  existing
  mod.rs
  map_report.rs               new
  map_assertions.rs           new
  wfc_trace.rs                new
  debug_map.rs                new

tests/
  map_contracts.rs            new
  map_diversity.rs            new
  map_regression.rs           optional
```

`src/testing/mod.rs` should re-export the helpers:

```rust
pub mod debug_map;
pub mod map_assertions;
pub mod map_report;
pub mod wfc_trace;
```

---

## Map Report Contents

A failed map assertion should produce at least these sections:

### 1. Configuration

```text
seed: 12345
width: 14
height: 10
theme: industrial
room size range: 4..=10
```

### 2. Debug Map

Use distinct glyphs for diagnosis, not player-facing aesthetic:

```text
.  floor
#  wall
+  door
*  conduit
~  pipe
N  north wall
S  south wall
E  east wall
W  west wall
7  north-east corner
8  north-west corner
9  south-east corner
0  south-west corner
space  void/deadzone
```

Example:

```text
  01234567890123
0 0NNNNNNNNNNNN7
1 W...........E
2 W...**......E
3 W...........E
4 W...........E
5 0SSSSSSSSSSSS9
```

Coordinates must be included for easy reference.

### 3. Passability Map

A separate grid showing only:

```text
. passable
# blocked
+ door
```

Example:

```text
  01234567890123
0 ##############
1 #...........#
2 #...........#
3 #....#......#
4 #...........#
5 ##############
```

This makes connectivity failures easy to inspect.

### 4. Room Graph Dump

```text
Room 0 Storage      [0,0 4x3]
Room 1 Reactor      [5,0 4x3]
Edge 0-1 Door East

System graphs:
electrical: 1 -> 0
```

### 5. WFC Trace / Domains

If WFC is used, print:

- initial domain restrictions per cell
- collapse order
- selected tile names
- propagated constraints
- contradiction cell
- domain before contradiction
- allowed set after important neighbor constraints
- why the domain became empty

Example:

```text
WFC contradiction at (5,2)

cell domain before contradiction:
  Floor, Conduit, Pipe, DoorNorth, DoorSouth

after neighbor (4,2)=WallEast:
  allowed = Floor, Conduit, Pipe

after neighbor (5,3)=WallNorth:
  allowed = EMPTY

last 8 collapses:
  (4,2) WallEast
  (5,3) WallSouth
  ...
```

This is the most useful section for an LLM.

### 6. Invariant Summary

Print each condition and whether it passed:

```text
[PASS] dimensions
[PASS] all required room types present
[FAIL] passable connectivity
[FAIL] no orphan doors
[PASS] system density
```

---

## Suggested Assertions

### Structural

- `size(expected_width, expected_height)`
- `no_orphan_floor()`
- `passable_connected()`
- `room_graph_connected()`
- `no_orphan_doors()`
- `door_walls_parallel()`
- `walls_have_valid_neighbors()`

### Room Types and Progression

- `has_room_type(RoomType)`
- `room_type_count(RoomType, range)`
- `room_count_between(range)`
- `has_room_with_flags(SystemFlags)`
- `room_has_system(RoomType, SystemFlags)`

### System Properties

- `system_density(SystemFlags, range)`
- `has_terminal_for_every_system_room()`
- `system_graph_connected_for(SystemFlags)`

### Local WFC / Pattern Checks

- `forbidden_2x2(patterns)`
- `forbidden_3x3(patterns)`
- `max_wall_run(length)`
- `max_door_count_per_room(count)`

---

## Diversity Tests

Do not require every seed to produce a completely different layout. Instead,
compute a compact signature from each generated map and assert a healthy
spread of signatures.

Example signature values:

- room count
- room size histogram
- wall direction histogram
- door direction histogram
- deadzone ratio
- diagonal wall ratio
- system tile ratio
- room graph degree distribution

Test example:

```rust
#[test]
fn different_seeds_produce_noticeably_different_maps() {
    let maps: Vec<_> = (0..25).map(generate_from_seed).collect();
    let unique_signatures = maps.iter().map(map_signature).collect::<HashSet<_>>();

    assert!(
        unique_signatures.len() >= 20,
        "only {}/{} unique signatures, generation is too samey",
        unique_signatures.len(),
        maps.len()
    );
}
```

This catches generation that looks different visually but still behaves like
the same layout.

---

## Human + LLM Workflow

### Human Developer

1. Write a failing map contract test.
2. Use explicit, map-level assertions.
3. Run:

```bash
cargo test map_contracts -- --nocapture
```

4. Capture the failed output.
5. Copy the complete report into the LLM/coding request.

### LLM / Coding Agent

1. Read the rich map report.
2. Identify which invariant or generation rule failed.
3. Modify the generator or WFC rule pack.
4. Run the same test.
5. Iterate until all map contract tests pass.

No map generator change should start from “looks boring” alone. It should
start from a failing test that demonstrates a measurable generation issue.

---

## First TDD Slices to Implement

1. Add debug map renderer.
2. Add basic `MapAssert`:
   - dimensions
   - required room types
   - room graph connected
3. Add WFC tile name mapping and contradiction trace.
4. Add passability flood fill and connectivity assertion.
5. Add no-orphan-door assertion.
6. Add system density assertions.
7. Add forbidden 2x2 pattern assertions.
8. Add room shape assertions once the rectangular-vs-nonrectangular decision is stable.
9. Add seed diversity test.

---

## Future Value

This approach turns procedural generation into a debug-friendly, testable
engineering problem.

It supports:

- new ship themes via rule packs and tests
- new room types via contract assertions
- new visible system tiles via density and local pattern checks
- reproduction of any map bug as a fixed-seed regression
- automated playtesting and MAP-Elites without a game client
- Eldiron-authored rule packs that can be validated by the same map contracts

This is the foundation for keeping procgen both expressive and reliable.
