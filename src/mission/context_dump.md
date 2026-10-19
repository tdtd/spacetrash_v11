# Context Dump Before Clearing

This file is a snapshot of the current development context, decisions, and
open directions at the point it was written. It is intentionally broad.

## 1. Current Game and Engine

- Name: Spacetrash v11
- Engine: Bevy 0.13
- Physics: custom arcade physics on the XZ plane
- Rendering: 3D visuals for top-down gameplay
- UI: Bevy UI overlays with a dedicated 2D UI camera
- Content/lore inspiration:
  - Duskers
  - Factorio
  - FTL: Faster Than Light

The project is still an early prototype, but the architecture is gradually
becoming graph + WFC + deterministic tests + diegetic UI.

---

## 2. Solved Architectural Direction

The current core mental model is:

```text
planar room/skeleton graph
        ↓
non-local graph constraints
        ↓
theme-specific WFC rule pack
        ↓
per-cell semantic domains
        ↓
seeded WFC collapse
        ↓
validation / rejection / retry
        ↓
concrete tile map
        ↓
spawn entities and system metadata
```

Important correction: the planar graph is not derived from tiles after WFC.
The planar graph is the **non-local skeleton**. WFC realizes that skeleton as
a concrete tile map.

Rooms do not have to remain rectangles forever. The current prototype still
uses rectangular rooms, but future themes may allow near-rectangular,
derelict, alien, or research-style irregular shapes.

---

## 3. Current Implementation State

### 3.1 Core Rust / Bevy state

Working:

- `cargo run`
- `cargo test`
- `sh test.sh` / `cargo test --all-targets`

Current Bevy app flow:

```text
Loading -> MainMenu -> Playing
```

There are also `Paused`, `GameOver`, and `Victory` states.

### 3.2 Components

Defined in `src/components/mod.rs`:

- `Velocity`
- `Rotation`
- `AngularVelocity`
- `AabbCollider`
- `CircleCollider`
- `ActorSize`
- `PhysicsBody`
- `Player`
- `Controllable`
- `Blackbox`
- `Drone`
- `FirstPersonCamera`
- `ActiveWorldCamera`
- `Ship`
- `Room`
- `DockingPort`
- `GridCoordinates`
- `WallSegment`
- `WallSegmentCollider`
- `TileNode`
- `GameplayEntity`

The most important recent component is `WallSegmentCollider`, which supports
inward-facing directional walls.

### 3.3 Resources

Defined in `src/resources/mod.rs` and related modules:

- `PhysicsConfig`
- `InputConfig`
- `EditorMode`
- `EditorTool`
- `DiegeticView`
- `SchematicLayer`
- `SchematicLayers`
- `FocusedEntity`
- `CurrentRoomGraph`
- `CurrentSystemGraphs`

Ship/scenario data lives under:

- `src/resources/ship_template.rs`
- `src/resources/scenario.rs`
- `src/resources/procgen.rs`

Current default ship template comes from `default_ship_template()`, not yet
from Eldiron assets.

### 3.4 Room and scaffold model

Defined in `src/procgen/scaffold.rs`:

- `RoomGraph`
- `RoomNode`
- `RoomEdge`
- `RoomType`
- `SystemFlags`
- `Rect`

Current `RoomType`s:

- `Storage`
- `Engineering`
- `Reactor`
- `LifeSupport`
- `Hydroponics`
- `ComputerCore`
- `Bridge`
- `Airlock`

System flags:

- `ELECTRICAL`
- `THERMAL`
- `ATMOSPHERICS`
- `FLUIDICS`
- `COMPUTER`

### 3.5 Tile model

Defined in `src/procgen/tile.rs`:

- `WallDirection` with 8 directions
- `TileType`:
  - `Floor`
  - `Wall(WallDirection)`
  - `Door(WallDirection)`
  - `Conduit`
  - `Pipe`

All visible tile types derive `PartialEq`, `Eq`, `Hash`, `Serialize`,
`Deserialize`.

### 3.6 WFC core

Defined in `src/procgen/wfc.rs`:

- `TileId`
- `Direction`
- `TileAdjacency`
- `WfcTileSet`
- `WfcSettings`
- `WfcBalanceProfile`
- `WfcGrid`
- `WfcResult`
- `run_wfc`
- `run_wfc_with_settings`

The solver supports:

- deterministic smallest-tile collapse
- seeded weighted collapse
- local adjacency propagation
- per-cell initial domains

`WfcGrid::new_with_domains` is important for graph-constrained WFC.

### 3.7 Graph-enhanced WFC

Defined in `src/procgen/graph_enhanced_wfc.rs`.

Current tile IDs:

```text
0 = Floor
1 = Conduit
2 = Pipe
3 = WallNorth
4 = WallSouth
5 = WallEast
6 = WallWest
7 = WallNorthEast
8 = WallNorthWest
9 = WallSouthEast
10 = WallSouthWest
11 = DoorNorth
12 = DoorSouth
13 = DoorEast
14 = DoorWest
15 = Void
```

Domain preparation currently:

- locks room boundary wall directions
- locks corners to proper `Wall(NorthWest)` etc.
- allows interior cells to be among `Floor`, `Conduit`, `Pipe`
- filters `Conduit` and `Pipe` according to room system flags
- locks door midpoint cells
- opens adjacent floor/pipe/conduit cells near doors

The actual collapse is now seeded weighted WFC, replacing the earlier
deterministic placeholder.

### 3.8 Semantic WFC wrapper

Defined in `src/procgen/semantic_wfc.rs`.

Current themes:

- `Industrial`
- `Derelict`
- `Military`

Current `WfcRulePack` fields:

- `name`
- `theme`
- `wall_style`
- `allows_concave_corners`
- `door_density`
- `deadzone_density`
- `wall_thickness_hint`
- `diagonal_wall_ratio`
- `system_tile_ratio`

This wrapper currently delegates to `graph_enhanced_collapse`. It is the
intended future home for more expressive rule-pack-driven generation.

---

## 4. Ship Spawning

Defined in `src/systems/ship_generation.rs`.

Current spawn process:

1. Build a deterministic planar `RoomGraph`.
2. Create variable column widths and row heights.
3. Choose room types with a simple seeded Fisher-Yates shuffle.
4. Add door edges between adjacent rooms.
5. Run `graph_enhanced_collapse`.
6. Insert:
   - `CurrentRoomGraph`
   - `CurrentSystemGraphs`
7. Optionally add extra random conduit/pipe decorations to rooms with the
   matching `SystemFlags`.
8. Spawn per-tile entities.

Spawn behavior by tile:

- `Floor`: floor plane.
- `Conduit`: floor plane + thin ceiling conduit line.
- `Pipe`: floor plane + thin floor pipe line.
- `Door`: floor plane + two side walls forming a passage.
- `Wall`: floor surface + directional wall slab + `WallSegmentCollider`.

Known visual limitation:
- wall slabs and door side walls are still very simple.
- later visual pass needed.

---

## 5. Map Assertions and TDD Work

The project now has rich map-level test helpers.

Current files:

- `src/testing/map_report.rs`
- `src/testing/map_assertions.rs`
- `src/testing/wfc_trace.rs`
- `src/testing/debug_map.rs`
- tests:
  - `tests/map_contracts.rs`
  - `tests/map_diversity.rs`
  - `tests/map_system_traces.rs`
  - existing tests for physics, WFC, scaffold, layered constraints, etc.

Map assertions supported so far:

- `size`
- `no_orphan_floor`
- `passable_connected`
- `room_graph_connected`
- `no_orphan_doors`
- `has_room_type`
- `room_count_between`
- `room_has_system`
- `system_density`
- `system_line_connectivity`
- `forbidden_2x2`

A failing map test prints:

```text
[FAIL] MapAssert test-name

Map:
  debug map

Passability:
  passability map

Invariant summary:
  PASS/FAIL lines
```

This is designed for human-to-LLM TDD workflows.

---

## 6. Recent Changes and Fixes

The most recent implementation work included:

1. Added `WallSegmentCollider` and circle-vs-wall-segment resolution.
2. Added 8-direction wall generation and tests.
3. Improved wall visuals from full cubes to thin outward-facing slabs.
4. Added floor surfaces under wall tiles.
5. Added door side walls.
6. Added ceiling conduit and floor pipe visibility.
7. Added graph-aware WFC interior domains:
   - electrical rooms can produce `Conduit`
   - fluidic rooms can produce `Pipe`
8. Added seeded weighted WFC in `graph_enhanced_collapse`.
9. Added map assertion helper modules.
10. Added map system trace tests.
11. Fixed several compile issues around Bevy image and sampler APIs.
12. Disabled doctests in `Cargo.toml` to avoid rustdoc check-cfg failure.
13. Added current mission/context docs.

Current tests all pass with `cargo test`, except the pre-existing unused
import warning in `tests/layered_constraints.rs` for `PathRequirement`.

---

## 7. Current Known Issues and Limitations

- The generator still creates a rectangular room grid. It is not yet a pure
  non-rectangular WFC room discoverer.
- Theme rule packs are only stubbed and do not yet strongly change map
  personality beyond parameters.
- System line placement is partially graph-aware, but not yet ideal:
  - conduits/pipes can still appear as isolated noise
  - no actual line connectivity between two specific system rooms is
    guaranteed end-to-end
- Docking merge is still data-only; no runtime graph merge.
- No enemy AI.
- No ship-system simulation.
- No ray/sensor grid.
- No research/tech nodes.
- Eldiron asset loading not implemented.
- In-game editor is mostly UI stub; no real world editing.
- Schematic UI modes are still provisional pixel buffers, not final vector
  or terminal modes.
- `WallDirection` diagonal variants are used in tests, but generator still
  mostly creates orthogonal room boundaries.

---

## 8. Immediate Next Steps

Priorities:

1. Strengthen `WfcRulePack` further and use it to shape WFC adjacency,
   weights, and validation.
2. Add more graph-aware system-line rules and TDD assertions for
   traceable conduit/pipe chains.
3. Add theme personality tests.
4. Add WFC trace/contradiction dump into map report.
5. Add first world critics:
   - connectivity
   - door quality
   - system trace
   - theme personality
6. Add Eldiron export/import later after code-level generation is stable.
7. Start runtime docking graph merge as a stubbed separate system.
8. Continue map contracts as the primary feedback loop.

---

## 9. Long-Term Plans / Daydreams

The project wants to move from fixed rooms to a more flexible generation
language.

Planned pipeline:

```text
planar skeleton graph
        ↓
non-local constraints
        ↓
theme rule pack
        ↓
semantic WFC domains
        ↓
seeded collapse
        ↓
validation / reject
        ↓
concrete tile map
        ↓
spawn world
```

Future ideas:

- Non-rectangular rooms behind rule-pack flags.
- Derelict, alien, research, military, quarantine, reactor-complex themes.
- System conduit/pipe/duct/network lines as physically visible, traceable
  world detail.
- Manual tracing without bridge/computer access.
- Room types as progression gates.
- Cascading ship-system failures inspired by FTL.
- Fluidics and atmospherics simulations.
- Upgrade-driven actor generation for player and enemies.
- Single ray-instrument constraint for sensors and cameras.
- Cellular-automaton sensor/visibility grid.
- Automated headless arena playtesting.
- MAP-Elites search over actor upgrades, layouts, and themes.
- World critics and map autopsy as first-class tools.
- Eldiron-authored rule packs, scenarios, upgrade definitions, and map
  contracts.

---

## 10. Relevant Files

### Procgen / WFC

- `src/procgen/scaffold.rs`
- `src/procgen/tile.rs`
- `src/procgen/wfc.rs`
- `src/procgen/graph_enhanced_wfc.rs`
- `src/procgen/layer.rs`
- `src/procgen/semantic_wfc.rs`
- `src/procgen/mod.rs`

### Resources

- `src/resources/mod.rs`
- `src/resources/ship_template.rs`
- `src/resources/scenario.rs`
- `src/resources/procgen.rs`

### Systems

- `src/systems/mod.rs`
- `src/systems/collision.rs`
- `src/systems/setup_scene.rs`
- `src/systems/ship_generation.rs`

### UI

- `src/ui/mod.rs`
- `src/ui/loading.rs`
- `src/ui/main_menu.rs`
- `src/ui/pause.rs`
- `src/ui/game_over.rs`
- `src/ui/hud.rs`
- `src/ui/editor.rs`
- `src/ui/command_line.rs`
- `src/ui/schematic.rs`
- `src/ui/config_view.rs`

### Testing

- `src/testing/mod.rs`
- `src/testing/map_report.rs`
- `src/testing/map_assertions.rs`
- `src/testing/wfc_trace.rs`
- `src/testing/debug_map.rs`

### Map tests

- `tests/map_contracts.rs`
- `tests/map_diversity.rs`
- `tests/map_system_traces.rs`

### Other tests

- `tests/actor_size.rs`
- `tests/collision.rs`
- `tests/gewfc.rs`
- `tests/layered_constraints.rs`
- `tests/movement.rs`
- `tests/scaffold.rs`
- `tests/test_harness.rs`
- `tests/tile.rs`
- `tests/wfc.rs`
- `tests/wfc_settings.rs`

### Authoring / mission docs

- `src/mission/index.md`
- `src/mission/procedural_generation.md`
- `src/mission/physics.md`
- `src/mission/rays.md`
- `src/mission/ai.md`
- `src/mission/ecs.md`
- `src/mission/eldiron.md`
- `src/mission/session_architecture.md`
- `src/mission/schematicsUI.md`
- `src/mission/context_dump.md` (this file)

### Design / meister docs

- `src/mien/design.md`
- `src/mien/duskers.md`
- `src/mien/upgrades.md`
- `src/meister/architecture.md`
- `src/meister/bevy.md`
- `src/meister/game_systems.md`
- `src/meister/tdd.md`
- `src/meister/eldiron.md`
- `src/mein/systems.md`
- `src/mein/systems.md`

---

## 11. Recommended Immediate Commit Message

If committing this context dump:

```text
docs: dump current procgen and TDD context
```

---

## 12. Final Note Before Context Clear

The current project state is stable enough to continue from this document
alone. The next useful work is very likely one of:

- strengthen map contract diagnostics
- make system lines more traceable
- make WFC rule packs more distinct
- begin runtime docking graph merge

Any of these can be picked up after the context is cleared.
