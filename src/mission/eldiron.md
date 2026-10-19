# Eldiron Integration Plan

## Goal

Use Eldiron as the **authoring/data layer** for ships, rooms, scenarios,
tiles, systems, upgrades, rules, and player-facing text. Bevy remains the
runtime and simulation owner.

Eldiron should not execute game simulation. It should export structured data
that the Bevy runtime loads into existing ECS resources.

---

## Current Repository Boundaries

The repository already has the correct serde-oriented shapes for this:

### Ship and Scenario Data

- `src/resources/ship_template.rs`
  - `ShipTemplate`
  - `Scenario`
  - `ShipInstanceInit`
  - `DockingLink`
  - `DockingPortDef`

- `src/resources/scenario.rs`
  - `ShipTemplateLibrary`
  - `ActiveScenario`

- `src/resources/procgen.rs`
  - `ProcGenContext`
  - `ShipGenOutput`
  - `ProcGenRegistry`

The current game hardcodes one template through `default_ship_template()`.
The most important first step is to replace that with Eldiron-exported assets.

---

## Core Strategy

1. **Eldiron authors data and assets.**
2. **Bevy loads those assets at startup.**
3. **Bevy constructs ECS entities from the loaded definitions.**
4. **The in-game editor exports back to Eldiron-compatible JSON.**

No logic or behavior should move into Eldiron itself. Eldiron should own
structure, balance, text, and asset references.

---

## Recommended Work Streams

### 1. Load Eldiron-Authored Ship Templates and Scenarios

Add asset-loading support for the existing serde structs.

Asset layout:

    assets/
      templates/
        test_ship.json
        derelict_01.json
      scenarios/
        default.json
        multi_ship.json

At startup, populate `ShipTemplateLibrary` from `assets/templates/`.
The hardcoded `default_ship_template()` should eventually be only a test
fallback or removed entirely.

A scenario file will spawn one or more ships and link their docking ports.

Example scenario JSON:

    {
      "ships": [
        {
          "template_id": "test_ship",
          "position": [0, 0],
          "rotation": 0.0
        },
        {
          "template_id": "derelict_01",
          "position": [30, 0],
          "rotation": 0.0
        }
      ],
      "docking_links": [
        {
          "ship_a": 0,
          "port_a": 0,
          "ship_b": 1,
          "port_b": 0
        }
      ]
    }

This maps directly onto the existing `Scenario` and `ShipInstanceInit`
types.

---

### 2. Author the Planar Room Graph in Eldiron

Eldiron regions, named sectors, and connections map naturally onto the
current room scaffold.

Mapping table:

| Eldiron concept | Spacetrash runtime concept |
| --- | --- |
| Named region/sector | `RoomNode` |
| Rectangular sector bounds | `Rect` |
| Named linedef / door / airlock | `RoomEdge` |
| Sector room type metadata | `RoomType` |
| Entity/item docking port | `DockingPortDef` |

Suggested authoring conventions:

- Sector names:
  - `Reactors`
  - `Bridge`
  - `LifeSupport`
  - `Storage`
  - `Airlock`

- Sector metadata:
  - `room_type`
  - `system_flags`

- Named connections:
  - `Door`
  - `Corridor`
  - `Airlock`
  - `Umbilical`

Eldiron can export these as a JSON `ShipTemplate` with:

- `manual_rooms`
- `door_overrides`
- `docking_ports`

The current Bevy generator already supports these fields; they are just not
loaded yet.

---

### 3. Use Eldiron Tile Metadata for WFC Interiors

Eldiron tile metadata should drive the currently stubbed WFC interior
generation.

Useful Eldiron tile metadata:

- procedural `kind`: `floor`, `wall`, `entrance`, `exit`
- `style`
- `weight`
- `blocking`
- `gameplay_tags`
- material `preset`
- material `finish`

Mapping to the current WFC model:

| Eldiron tile data | Spacetrash WFC data |
| --- | --- |
| Authored tile UUID | `TileId` |
| Tile adjacency rules | `WfcTileSet.adjacency` |
| `weight` | `WfcBalanceProfile.tile_weight_multipliers` |
| Material preset/finish | Bevy material/color/roughness |
| `gameplay_tags` | future tile behavior tags |

Current `src/procgen/wfc.rs` already supports:

- local adjacency constraints
- seeded weighted collapse
- custom balance profiles

The missing part is mostly parsing Eldiron tile-set definitions into these
Rust types.

---

### 4. Author Ship Systems and System Graphs

The existing `SystemFlags` and `RoomType` types are a good start.

Eldiron should author system flags as metadata for each room.

Example TOML-like authoring snippet:

    [room]
    room_type = "Reactor"
    system_flags = ["ELECTRICAL", "THERMAL"]

Existing room-type to system-flag mappings:

| Room type | System flags |
| --- | --- |
| `Engineering` | `ELECTRICAL` |
| `Reactor` | `ELECTRICAL | THERMAL` |
| `LifeSupport` | `ATMOSPHERICS` |
| `Hydroponics` | `FLUIDICS | ATMOSPHERICS` |
| `ComputerCore` | `COMPUTER` |
| `Bridge` | `COMPUTER | ELECTRICAL` |
| `Airlock` | none |

From those flags, the current `derive_graph_constraints()` can create layer
requirements. Those constraints can later become dedicated `SystemGraph`
resources for:

- Electrical
- Fluidics
- Atmospherics
- Thermal
- Computer
- Docking
- Sensors
- Research

---

### 5. Author Upgrades, Behaviors, and Rules as Data

The design documents describe an upgrade-driven actor system where:

> An actor is the sum of its upgrades.

Eldiron can author:

- Chassis definitions
- Upgrade modules
- Movement, sensor, effector, utility, and behavior upgrades
- Ray-instrument flags
- Behavior FSA transition tables

Bevy should load these into an `UpgradeRegistry` and use an actor builder to
assemble both player drones and NPCs.

Example upgrade definition:

    {
      "id": "visible_camera",
      "category": "sensor",
      "ray_instrument": true,
      "components": [
        "LightReceiver",
        "VisibleCameraView"
      ]
    }

The runtime should map those definitions to actual ECS components.

It should not import Eldiron's script VM.

---

### 6. Use Eldiron for Locale, Rules, and Audio Data

Current UI text is largely hardcoded. Eldiron should export:

- Locale tables
- Room/system names
- Upgrade descriptions
- Research node text
- Message templates
- Audio FX references

Add lightweight Bevy resources:

    struct LocaleTable(HashMap<String, String>);
    struct RulesetData(serde_json::Value);

UI systems should read strings from the locale resource instead of
hardcoding them.

This keeps Bevy code clean while Eldiron owns player-facing text and balance.

---

### 7. Round-Trip In-Game Editor with Eldiron

The in-game editor is conceptually Factorio-like and should edit the same
data structures that Eldiron edits.

If the in-game editor saves:

- Rooms
- Wall/door changes
- Docking ports
- Ship templates
- Scenario composition

using the same `ShipTemplate` and `Scenario` JSON shapes, those files can
later be opened in Eldiron Creator.

Workflow:

    Eldiron Creator authoring
            ↓ export JSON/TOML
    Bevy runtime loading
            ↓ editor/god mode changes
    Back to Eldiron-compatible JSON

The current `src/mandate/editor_bdd.md` already describes editor behavior.
The next step is serializing those edits into the same serde models.

---

### 8. Use Eldiron Procedural Recipes for Future Visual Generation

Eldiron docs include:

- Builder Graph
- Procedural Recipes
- SDF Shapes
- Tile Node Graphs
- Material recipes
- Headgear/wearable SDFs

These are useful later for:

- Deterministic ship-template interiors
- Authored WFC tile sets with procedural textures
- Enemy/item appearance generation
- UI iconography and layered schematic styles

Do not integrate the recipe engine directly. Instead, export generated
artifacts as ordinary PNG textures or simple metadata into the Bevy asset
pipeline.

---

## Relevant Files

The following files are relevant to the Eldiron integration, listed as plain paths and short descriptions.

src/resources/ship_template.rs - ShipTemplate, Scenario, ShipInstanceInit, DockingLink, DockingPortDef

src/resources/scenario.rs - ShipTemplateLibrary, ActiveScenario

src/resources/procgen.rs - ProcGenContext, ShipGenOutput, ProcGenRegistry

src/procgen/scaffold.rs - RoomGraph, RoomNode, RoomEdge, RoomType, SystemFlags, Rect

src/procgen/wfc.rs - WfcTileSet, WfcConfig, WfcBalanceProfile, seeded weighted collapse

src/procgen/tile.rs - TileType, WallDirection

src/systems/ship_generation.rs - spawn_ship_from_template, SimpleRng, graph-to-world spawning

src/systems/mod.rs - system scheduling and generation integration points

src/ui/schematic.rs - layer-specific schematic UI and image generation

src/app.rs - app startup, resource registration, plugin ordering

src/testing/mod.rs - test harness and helper entity creation

tests/common/mod.rs - shared test setup

Cargo.toml - serde, bitflags, bevy_prototype_lyon

assets/templates/ - Eldiron-exported ship template files (future)

assets/scenarios/ - Eldiron-exported scenario files (future)

---

## Non-Goals

- No Eldiron script execution in Bevy.
- No move of game logic into Eldiron.
- No direct coupling of Bevy rendering to Eldiron runtime APIs.
- No binary asset coupling; serialized JSON/TOML and standard image files are
  the initial integration boundary.

---

## Result

Eldiron becomes the authoring front-end for:

- Ship layouts
- Room scaffolding
- WFC tile sets
- System graphs
- Upgrades
- Behaviors
- Locale and balance data

Bevy remains the simulation, rendering, input, and networking runtime.

The integration is achieved through the existing serde models and a small,
clean asset-loading bridge rather than through engine-level coupling.
