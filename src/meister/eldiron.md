# Eldiron Authoring and Data Integration

Eldiron is the intended authoring layer for **Spacetrash v11**. Bevy owns the real-time simulation, while Eldiron defines content such as ship scaffolds, scenario data, room types, system flags, WFC tile sets, and upgrade definitions.

## Current Role

Eldiron is not embedded as a runtime engine inside the Bevy project. Instead, Eldiron assets are exported into serializable formats that Bevy loads at startup or when opening a scenario.

This gives the project a clean boundary:

- Eldiron authors content
- Bevy executes the game
- the in-game editor can later save back to Eldiron-compatible data formats

## Authoring vs Runtime

The most important separation is between:

- **Ship Scaffold** (planar room graph)
- **Ship Instance** (runtime entity generated from scaffold)
- **Scenario** (which scaffolds exist and how they connect)
- **World State** (saved runtime data used when resuming a game)

The scaffold is authored data. The ship instance is a runtime entity generated from that scaffold. The world state is saved runtime data.

## Current Rust Data Structures

The prototype defines these as plain Rust structs with `serde` support:

- `src/resources/ship_template.rs`
  - `ShipTemplate`
  - `ManualRoom`
  - `DoorOverride`
  - `DockingPortDef`
  - `Scenario`
  - `ShipInstanceInit`
  - `DockingLink`
  - `WorldState`

- `src/resources/scenario.rs`
  - `ShipTemplateLibrary`
  - `ActiveScenario`
  - `default_ship_template`

These structures are designed to be produced by Eldiron and consumed by Bevy systems. They will evolve to represent planar scaffolds and WFC tile sets.

## Ship Scaffolds

A ship scaffold represents one reusable ship blueprint as a planar room graph.

It contains:

- stable ID and name
- procedural generation seed
- room nodes with room types and system flags
- room edges (door, corridor, airlock, umbilical)
- manual room overrides
- door overrides
- docking port definitions
- WFC tile set references per room type

This supports the core design goal of mixing procedural generation with manual edits.

## Scenarios

A `Scenario` represents the active playfield composition.

It contains:

- scenario ID and name
- list of ship scaffold instances
- optional initial docking links between scaffolds
- optional saved world state

This matches the design principle that there are no discrete levels. The playfield is the union of all currently loaded scaffolds, represented as a single typed room/tile graph.

## Procedural Generation Algorithms

Eldiron can later provide procedural recipes and generation data. The Bevy runtime uses a first-class algorithm registry.

The current Rust interface is defined in:

- `src/resources/procgen.rs`

Key types:

- `ProcGenId`
- `ProcGenContext`
- `ShipGenOutput`
- `ShipGenerationAlgorithm`
- `ProcGenRegistry`

The registry lets algorithms be selected by ID. The same scaffold can choose between a built-in Rust algorithm, a dynamically loaded algorithm, or, later, an adapter around Eldiron procedural recipes.

## Planned Eldiron Mapping

### Creator Geometry and Map Tools

Eldiron regions, sectors, linedefs, and direct geometry can later be exported as room scaffold nodes and edges.

The current Bevy world assumes:

- rectangular rooms
- axis-aligned walls
- 1-tile door openings
- docking ports on room boundaries

Eldiron's 2D/3D editing tools fit this room scaffold model.

### Builder Graph and Procedural Recipes

Eldiron Builder Graph and Procedural Recipes may supply:

- room interior detail via WFC tile sets
- wall attachments
- light and particle templates
- system port definitions

These would be consumed by the Bevy ship generator as authored room/interior templates.

### Rules, Locales, and Actor Upgrades

Eldiron can later author:

- drone and Blackbox upgrade modules
- behavior FSA definitions
- research/tech tree data
- localization strings for terminal/schematic overlays
- behavior transition rules

The Bevy runtime will deserialize these into components and resources rather than embedding Eldiron's script VM.

## In-Game Editor Relation

The in-game god-mode editor is inspired by Factorio's map editor.

The editor edits the live typed room/tile graph while simulation is paused. Later, it should save:

- ship layouts as scaffold data
- world composition as `Scenario`

Those saved structures are the same structures Eldiron can open and edit.

## Integration Phases

### Phase 1: Code-First Data

Current prototype uses Rust-defined default templates and scenario structs. No JSON or TOML parser is used yet.

### Phase 2: Serialized Assets

Add JSON or TOML loading for:

- `ShipTemplate` (scaffold data)
- `Scenario`

Define a simple asset folder layout:

```text
assets/
  templates/
    test_ship.json
  scenarios/
    default.json
```

### Phase 3: Eldiron Export

Use Eldiron Creator to author ship scaffolds and scenarios. Export those files into the above layout.

### Phase 4: Procedural Algorithm Bindings

Map Eldiron procedural data or scripts into `ShipGenerationAlgorithm` implementations, registered by ID.

### Phase 5: Save/Reload Round Trip

Let the in-game editor save back into formats that Eldiron can open and continue editing.

## Automated Testing Data Requirements

For automated playtesting, Eldiron-exported data must be deterministic and
parameterizable.

### Configuration Hash

Each exported ship scaffold, scenario, and upgrade definition should be
versioned and hashable. The hash becomes part of the arena configuration.

This allows exact reproduction of the same world/actor set across runs.

### Test Asset Package

Eldiron should support exporting “arena definition” packages that contain:

- a ship scaffold or scenario
- initial ship system states
- actor upgrade profiles
- enemy generation tables
- WFC interior tile sets
- seed values

Bevy loads these packages into a headless arena runner.

### Data Validation

Before an automated run, validate exported data:

- all referenced scaffold IDs exist
- docking links are valid
- scaffold graph is connected
- all room types have valid system flags
- actor upgrade definitions map to known components
- no ray instrument conflicts
- no procedural generation parameters out of range
- WFC tile sets are available for all referenced room types

Invalid arena definitions are rejected.

## Boundary Rules

To avoid coupling issues:

- Bevy must not depend on Eldiron runtime code.
- Eldiron must not depend on Bevy ECS types.
- Shared data types should remain plain Rust with serde.
- Eldiron may use its own scripting and rules format internally.
- Bevy translates exported data into the typed room/tile graph at world build time.

This boundary keeps Eldiron useful for authoring while keeping Bevy performant and focused on real-time simulation.

## Future Automation Integration

Eldiron could eventually include an export target specifically for the
automated testing arena, such as:

```text
arena/search_config.json
```

This file describes search space ranges, candidates, and evaluation
descriptors. Bevy’s search runner can read it and execute MAP-Elites or
random sampling.

This is a long-term goal, not a Phase 1 requirement.
