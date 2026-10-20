# Spectator Mode Mission Plan

## Purpose

This mission plan describes the actionable implementation steps for adding a
**spectator mode** to Spacetrash v11.

Spectator mode is the rendered, interactive counterpart to the existing
headless deterministic arena. It must use the same `ArenaConfig`, world
builder, simulation systems, event log, and map assertions as the automated
tests.

The initial goal is a developer debug tool. The long-term goal is a
spectator/replay system that can also feed into the in-game editor.

---

## Current State

The project already has:

- `src/arena/config.rs`
  - `ArenaConfig`
  - `ArenaActorSpawn`
  - `ArenaShipConfig`

- `src/arena/event.rs`
  - `ActorState`
  - `ArenaEvent`
  - `ArenaEventLog`
  - `ArenaSnapshot`

- `src/arena/runner.rs`
  - `build_arena_app`
  - `run_arena`

- `src/arena/world_builder.rs`
  - `build_arena_world`
  - `reset_arena_world`
  - `generate_ship_data`
  - `spawn_arena_actor`

- `src/testing/mod.rs`
  - `test_app`
  - `run_n_ticks`
  - `press_key`
  - deterministic headless test utilities

- `src/testing/map_assertions.rs`
  - `MapAssert`
  - debug map and passability map output

- `src/systems/ship_generation.rs`
  - full render-capable ship spawner

- `src/ui/editor.rs`
  - editor/godmode UI overlay

The gap is that there is no headful spectator system that renders the same
headless simulation.

---

## Goals

1. Reuse the headless arena runner for headful spectator mode.
2. Add a developer-only spectator UI with play/pause/step/speed/scrob controls.
3. Render map assertion overlays inside the spectator view.
4. Support loading the same test fixtures used by `cargo test`.
5. Eventually allow spectator-to-godmode handoff for editing.

---

## Non-Goals

- No public player-facing replay system yet.
- No world mutation in spectator mode.
- No separate `GameState` for spectator.
- No simulation changes that affect `run_arena` output.
- No audio/particle polish in spectator mode initially.
- No multiplayer spectating yet.

---

## Implementation Phases

### Phase 1 — Refactor Arena Runner into Shared Core

**Goal:** Extract a simulation core that can be used both headlessly and in
spectator.

#### Work

- Define `SimulationSession` and related enums in a new module, likely
  `src/arena/session.rs`.
- Move fixed-tick clock, event recording, and world reset into the session.
- Create a `SimulationPlugin` that registers:
  - behavior arbitration
  - movement system
  - arena snapshot recording
- Keep `run_arena` as a thin wrapper around the shared plugin/session.

Suggested module:

```text
src/arena/session.rs
```

Suggested resources:

```rust
pub struct SimulationSession {
    pub config: ArenaConfig,
    pub tick: u64,
    pub state: SimulationState,
    pub speed: SimulationSpeed,
    pub event_log: ArenaEventLog,
    pub final_snapshot: Option<ArenaSnapshot>,
}
```

```rust
pub enum SimulationState {
    Stopped,
    Running,
    Paused,
    Finished,
}
```

```rust
pub enum SimulationSpeed {
    Paused,
    Normal,
    Fast,
    Step,
}
```

#### Tests

- `same_config_old_runner_equals_new_session`
- `simulation_session_tick_records_event`
- `speed_step_advances_exactly_one_tick`
- `paused_session_advances_zero_ticks`

#### Acceptance

- `run_arena(config)` still produces the same `ArenaRunResult`.
- `SimulationSession::tick()` is deterministic and matches old `run_arena` when
  run for the same number of ticks.

---

### Phase 2 — Add Basic Spectator Mode

**Goal:** Allow launching a spectator session from the running game and
watching a hardcoded arena scenario.

#### Work

- Add `SpectatorMode` resource.
- Add `SpectatorSession` resource that owns current config and session state.
- Add a dev key or menu action to enter spectator.
- Build world with same `build_arena_world` used by headless runner.
- Attach visual meshes/materials/cameras from the normal render path.
- Add a simple follow camera that focuses one actor.
- Add play/pause/step controls.

Recommended entry point:

- Keyboard shortcut: `F10` in debug builds
- Terminal command: `spectator start <fixture-id>`

Do not use `F10` permanently; it is reserved for schematic layers. Mark the
binding as temporary and dev-only until better UX exists.

#### Spectator Resource

```rust
#[derive(Resource)]
pub struct SpectatorMode {
    pub active: bool,
    pub config: ArenaConfig,
    pub session: SimulationSession,
    pub show_overlays: bool,
}
```

#### UI

Add a simple spectator panel with:

- current tick
- total tick count
- state
- speed
- buttons:
  - `Play`
  - `Pause`
  - `Step`
  - `Speed+`
  - `Speed-`

#### Camera

Initial spectator camera can follow the first actor in the arena. Later,
allow selecting actor focus via number keys or UI.

#### Acceptance

- A developer can start the game, enter spectator mode, and watch a generated
  arena run automatically.
- The same world is generated as in the headless runner.
- Play/pause/step do not disturb normal live play when spectator is inactive.

---

### Phase 3 — Render Arena Event Log and Map Overlays

**Goal:** Make the same TDD diagnostic data visible in spectator.

#### Work

- Render tail of `ArenaEventLog` in a spectator terminal-like panel.
- Toggleable `MapAssert` overlays:
  - debug tile map
  - passability map
  - failed invariants summary
- Highlight currently focused actor.
- Show per-actor `AiBehaviorState` in a small list.

The map overlay can be drawn using the existing `debug_map` and
`render_passability_map` functions from `src/testing/debug_map.rs`.

These render as text panels or textured sprites for the spectator view.
Text panels are simpler.

#### Acceptance

- A failing map assertion can be inspected visually in spectator.
- The displayed passability map matches `map_assertions` output.
- The event log tail updates as simulation runs.

---

### Phase 4 — Replay and Scrubbing

**Goal:** Allow stepping backward through recorded `ActorState` snapshots.

#### Work

- Store `ActorState` snapshots from each `ArenaEvent::TickSnapshot`.
- Add a timeline slider.
- When scrubbing:
  - pause simulation
  - render world proxies from selected snapshot
- Add `Now` button to return to latest live tick.

#### Data

Reuse `ArenaEventLog` and `ArenaSnapshot` already available in
`src/arena/event.rs`.

#### Acceptance

- A developer can scrub back and forth through a finished scene.
- Actor positions update from snapshots without rerunning simulation.
- Returning to `Now` resumes at the latest recorded tick.

---

### Phase 5 — Load Test Fixtures

**Goal:** Run the same TDD fixtures in spectator mode.

#### Work

- Define fixture path convention:

```text
tests/fixtures/arena/*.json
```

or

```text
assets/dev/arena/*.json
```

- Add `load_arena_config_from_json(path)`.
- Register a few built-in fixtures:
  - `two_room_map`
  - `four_sided_station`
  - `behavior_aggressive`
  - `movement_collision`

- Add terminal command:

```text
spectator load two_room_map
spectator load tests/fixtures/four_sided_station.json
```

- Share identical fixtures between `cargo test` and spectator mode.

#### Acceptance

- A checked-in test fixture runs in headless test and spectator without
  modification.
- Loading a JSON fixture into spectator produces the same initial world as the
  test.

---

### Phase 6 — Godmode Handoff

**Goal:** Allow a spectator scene to be inspected and edited in godmode.

#### Work

- Pause spectator session.
- Despawn spectator camera/UI, but keep the world entities.
- Set `EditorMode.active = true`.
- Preserve `CurrentRoomGraph`, `CurrentSystemGraphs`, and other resources.
- After edits, export updated `ArenaConfig` or `Scenario`.

#### Acceptance

- A spectator can pause, enter godmode, edit, exit, and resume spectating.
- Editing does not mutate `ArenaEventLog` except through intentional
  re-generation.
- Exporting a revised config produces a loadable fixture.

---

## Testing Plan

### Determinism

- `spectator_session_tick_matches_headless_runner`
- `same_seed_same_event_log_in_spectator_and_test`
- `ordered_actor_states_are_stable`

### World Reset

- `spectator_world_reset_is_leak_free`
- `resetting_spectator_preserves_nothing_from_previous_run`
- `spectator_and_headless_world_build_from_same_config_are_equal`

### Map Assertion Overlay

- `spectator_overlay_source_equals_map_assert_report`
- `failed_map_contract_can_be_displayed_in_spectator`

### UI / Mode

- `normal_input_is_disabled_in_spectator`
- `editor_input_is_disabled_in_spectator`
- `spectator_active_not_editor_active`
- `spectator_active_runs_simulation`

### Fixture Round Trip

- `serde_round_trip_arena_config`
- `test_fixture_json_loads_into_spectator`
- `spectator_config_can_be_exported`

---

## Required File Changes Summary

### New Files

- `src/arena/session.rs` — shared `SimulationSession`
- `src/systems/spectator.rs` — spectator mode systems
- `src/ui/spectator.rs` — spectator UI overlay
- `tests/spectator_mode.rs` — spectator tests
- `assets/dev/arena/*.json` or `tests/fixtures/arena/*.json` — shared fixtures

### Modified Files

- `src/arena/runner.rs` — use shared `SimulationSession`
- `src/arena/mod.rs` — export new session types
- `src/resources/mod.rs` — add `SpectatorMode`, `SimulationMode`
- `src/systems/mod.rs` — add run condition for spectator; route spectator
  systems
- `src/app.rs` — register spectator startup/update systems and dev bindings
- `src/ui/mod.rs` — add `spectator` module
- `src/testing/mod.rs` — add helper to create spectator test app from
  `ArenaConfig`

---

## BDD Scenarios

### Scenario: Spectator runs the same simulation as headless

- Given identical `ArenaConfig` seeds and tick counts
- When the simulation runs in headless and spectator modes
- Then the final snapshots match exactly
- And the `ArenaEventLog` length matches
- And actor positions/velocities match within floating-point epsilon

### Scenario: Spectator can be paused and stepped

- Given spectator mode is active and simulation is running
- When the player presses the `Pause` control
- Then the simulation state becomes `Paused`
- And no additional ticks occur
- When the player presses `Step`
- Then exactly one additional tick occurs
- And the tick counter increments by one

### Scenario: Spectator map overlays show the same data as MapAssert

- Given a generated arena map with a known map contract failure
- When the player toggles the map assertion overlay
- Then the overlay shows the same failed invariant as the headless `MapAssert`
  report
- And the passability map colors match the debug passability renderer

### Scenario: Spectator UI does not interfere with normal live play

- Given spectator mode is inactive
- And the game is in normal `Playing`
- Then normal movement, pause, and terminal input systems run normally
- And no spectator UI is visible
- And no spectator camera overrides the live camera

### Scenario: Spectator can load a test fixture

- Given a valid arena JSON fixture file exists
- When the developer loads it into spectator mode
- Then the world is rebuilt from that fixture
- And the initial tick, actors, and ship layout match the fixture
- And the same fixture can be run headlessly in a test

---

## Immediate Next Steps

Start with **Phase 1**.

The first executable milestone is:

```text
Extract SimulationSession without changing `run_arena` output.
```

Then add a dev-only spectator entrypoint.

Do not attempt replay scrubbing, JSON fixtures, or godmode handoff until the
basic spectator mode renders and runs the same simulation as headless.

---

## Summary

Spectator mode is the visual companion to the existing deterministic test
pipeline.

It gives the team a way to inspect map failures, behavior FSA decisions, and
docking scenarios in real rendering, using the exact same `ArenaConfig` and
event log as `cargo test`.

The implementation starts by refactoring the already simple headless arena
runner into a shared core, then incrementally adds UI, overlays, replay, and
editor handoff.

This will make Spacetrash development significantly easier by allowing a
human to see what the LLM sees, without breaking deterministic TDD guarantees.
