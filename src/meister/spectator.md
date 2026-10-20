# Spectator Mode and Headfull Replay Design

## Purpose

This document defines the **spectator mode** for Spacetrash v11.

Spectator mode is the rendered, interactive counterpart to the existing
headless deterministic test arena. It allows a human developer to watch,
pause, step, and inspect the same `ArenaConfig` scenarios that are used by
automated tests and LLM-driven map analysis.

Spectator mode starts as an internal debug tool, but it is designed to evolve
into a proper in-game replay/spectator system for:

- generated map contracts
- deterministic arena playthroughs
- behavior FSA analysis
- docking and multi-ship scenarios
- engine/editor handoff

---

## Core Principle

> Spectator is headful headless mode.

The simulation core must be identical between:

- `cargo test`
- headless `run_arena`
- headful spectator
- future LLM-analysis runs

Only the presentation and controls differ.

This prevents the common failure where a test passes headlessly but behaves
differently in the real engine, or where debug overlays require duplicating
simulation logic.

---

## Three Simulation Modes

There are three developer-facing world modes, in addition to normal live
play:

| Mode | Simulation runs? | Player input? | Editor mutation? | Output |
|---|---:|---:|---:|---|
| **Live play** | Yes | Yes | No | Real gameplay |
| **Editor / Godmode** | No | Only editor input | Yes | Live editable world |
| **Spectator** | Yes, deterministic | Playback controls only | No | Rendered simulation + event log |
| **Headless test** | Yes, deterministic | No | No | `ArenaRunResult` |

Live play is the normal game. Editor/godmode pauses simulation and allows
world editing. Spectator runs the same simulation as headless but renders it.

---

## Shared Core: `SimulationSession`

To avoid divergence, spectator and headless runners must share a common
simulation lifecycle object.

The recommended core resource is:

```rust
struct SimulationSession {
    config: ArenaConfig,
    tick: u64,
    state: SimulationState,
    speed: SimulationSpeed,
    event_log: ArenaEventLog,
    final_snapshot: Option<ArenaSnapshot>,
}
```

### State

```rust
enum SimulationState {
    Stopped,
    Running,
    Paused,
    Finished,
}
```

### Speed

```rust
enum SimulationSpeed {
    Paused,
    Normal,
    Fast,
    Step,
}
```

`ArenaConfig` already exists in `src/arena/config.rs`.

`ArenaEventLog` and `ArenaSnapshot` already exist in `src/arena/event.rs`.

Both headless and spectator drive the same `SimulationSession`.

---

## Simulation Plugin

The current headless arena runner assembles systems manually in
`src/arena/runner.rs`:

```text
build_arena_world
behavior_arbitration_system
movement_system
record_arena_snapshot
```

This should be extracted into a shared `SimulationPlugin`.

The plugin must:

1. Insert required resources.
2. Build or reset the world from `ArenaConfig`.
3. Run behavior arbitration.
4. Run deterministic movement and physics.
5. Record a structured `ArenaEvent::TickSnapshot`.
6. Advance the fixed simulation clock.

### Usage

Headless:

```rust
let app = build_headless_app(config);
run_ticks(app, config.tick_count);
```

Spectator:

```rust
let app = build_spectator_app(config);
```

Both use the same plugin.

---

## Determinism Boundaries

The following items must remain deterministic:

- `ArenaConfig`
- fixed tick clock
- behavior arbitration
- physics movement
- event log snapshot ordering
- entity spawning order

Rendering may use smoothed interpolation for presentation, but the source
state must still be taken from deterministic simulation tick state.

Unordered iteration must not leak into event ordering. The existing runner
sorts `ActorState` by entity index before recording snapshots. This property
must be preserved in spectator.

---

## Spectator Mode UI

The spectator UI is a developer instrument panel, not a player HUD.

It must expose:

- Current tick number
- Total planned tick count
- Simulation state: `Running`, `Paused`, `Finished`, etc.
- Play / Pause / Step controls
- Speed controls
- Timeline scrubber
- Active actor focus selector
- Per-actor behavior state
- Event log tail
- Map assertion overlay toggle
- Passability overlay toggle
- Room graph overlay toggle

Example compact layout:

```text
[SPECTATOR]
tick: 0421 / 1200
state: RUNNING
speed: NORMAL

>>  pause  step  speed+

Actor: Drone 1
Behavior: Scavenge

LOG
0401 Drone 1 entered Storage
0402 Drone 1 detected loot
```

This UI must be separate from the normal command line and schematic overlays.

---

## Map Assertions in Spectator

The existing `MapAssert` builder produces detailed reports for failing map
contracts.

In spectator, those same checks should be able to run live against the
current generated world and render:

- failed invariant summary
- debug tile map
- passability map
- orphan door markers
- disconnected passable component highlight
- room graph connectivity issues
- system line connectivity components

The ideal result is that a failing TDD map test can be opened inside the game
and inspected directly.

The source of truth remains `MapAssert`. Spectator only visualizes its data.

---

## Replay and Timeline

`ArenaEvent::TickSnapshot` already stores actor state for each tick.

Spectator can use these snapshots as replay data.

Two modes are possible:

1. **Live simulation** — current world continues advancing tick by tick.
2. **Recorded replay** — render already-captured `ActorState` snapshots
   without mutating the world.

### Scrubbing

A timeline scrubber allows moving backward and forward through recorded
ticks.

Interaction with live simulation:

- Scrubbing backwards pauses live simulation.
- Scrubbing forwards after a rewind can either:
  - replay recorded snapshots
  - resume simulation from the selected tick if deterministic snapshot
    restoration is available

For the first implementation, replay from recorded snapshots is sufficient.
Rerunning from an intermediate tick is a later optimization.

---

## Overlap with Godmode

Godmode and spectator share developer tool concerns but have different
purposes.

### Shared

- Both are modes inside `GameState::Playing`
- Both are dormant in normal player builds or dev-only
- Both may load `ArenaConfig`/scenario data
- Both benefit from map overlay debug drawing
- Both can eventually save/export revised configurations

### Distinct

| Concern | Godmode | Spectator |
|---|---|---|
| Simulation | Paused | Runs |
| Input | Edit floor/wall/door/dock | Transport controls |
| Camera | Free-fly editor | Follow selected actor or overview |
| Mutation | Direct world mutation | No mutation |
| Output | Editable world | Recorded run |

### Handoff

A useful future loop is:

```text
Spectator observes a generated scenario
        ↓
Pause spectator
        ↓
Enter godmode on the same world
        ↓
Edit room/door/dock/system data
        ↓
Export new ArenaConfig/scenario
        ↓
Run headless tests + spectator again
```

This requires spectator and editor to operate on the same world built from
the same `ArenaConfig`, not two separate spawn paths.

---

## Engine State Integration

`GameState` should not need a separate `Spectator` state.

Spectator should be a mode within `GameState::Playing`, like `EditorMode`.

### Proposed Resources

```rust
#[derive(Resource)]
struct SpectatorMode {
    active: bool,
    config: ArenaConfig,
    session: SimulationSession,
    show_overlays: bool,
}

#[derive(Resource, Default)]
struct SimulationMode {
    mode: SimulationModeKind,
}

enum SimulationModeKind {
    Live,
    Headless,
    Spectator,
    Editor,
}
```

`EditorMode` and `SpectatorMode` can remain separate resources, but both are
variants of a single **world mode** concept.

### Run Condition

Current `playing_and_not_editor` must account for spectator:

```rust
fn playing_and_live(
    state: Res<State<GameState>>,
    editor: Res<EditorMode>,
    spectator: Res<SpectatorMode>,
) -> bool {
    *state.get() == GameState::Playing
        && !editor.active
        && !spectator.active
}
```

This prevents:

- normal player input
- normal camera follow
- pause input
- debug game-over shortcut

from running while spectating.

---

## Sim Clock vs Bevy Time

Do not confuse Bevy `Time` with simulation ticks.

Headless runner currently does:

```rust
app.world.resource_mut::<Time>().advance_by(dt);
```

Spectator will render with Bevy `Time`, but simulation must use a separate
fixed accumulator.

Recommended:

```rust
struct FixedTickClock {
    accumulator: f32,
    tick: u64,
}
```

This ensures:

- deterministic simulation
- UI animation does not affect simulation
- same tick length as `PhysicsConfig::default().fixed_timestep`
- headless/headfull parity

---

## Expected Benefits

By building spectator on top of the existing TDD infrastructure:

- A map contract failure can be inspected visually.
- A behavior FSA issue can be watched frame by frame.
- A docking mismatch can be loaded as a multi-ship scenario.
- The LLM can request a `.arena.json` fixture from a failing test and inspect
  the same data a human sees.
- The jump from debug tool to spectator/replay feature is gradual and natural.

It also reduces duplicated spawn/simulation code.

---

## Non-Goals

- Spectator is not a player-facing cinematic mode yet.
- It does not edit the world.
- It does not give special gameplay powers.
- It does not alter normal game balance.
- It is not required for the initial public prototype.

It starts as a developer tool and may later be exposed as a limited
spectator/replay system.

---

## Summary

Spectator mode is the headful mirror of the headless deterministic arena.

It uses:

- the same `ArenaConfig`
- the same `ArenaEventLog`
- the same map assertions
- the same world builder
- the same simulation tick loop

The only additions are rendering, playback controls, debug overlays, and
optional scrubbing.

This approach turns already-existing tests and fixtures into visual,
inspectable scenes without sacrificing determinism.
