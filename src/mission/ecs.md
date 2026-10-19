# ECS Mission Plan

## Purpose

This document defines the focused implementation plan for the Bevy ECS
architecture used by **Spacetrash v11**. It expands the broad architecture
document into concrete scheduling, component, resource, and entity
responsibilities, now incorporating the typed room/tile graph, FSA runtime,
sensor grid, and upgrade-driven actor generation.

## Guiding Principles

- Bevy owns the runtime; Eldiron is only an authoring layer.
- Components are plain data.
- Resources store global configuration and runtime state.
- Systems are small and run only when relevant.
- UI overlays and diegetic interfaces must never directly manipulate world
  simulation unless through well-defined components/resources.
- The typed room/tile graph is the single source of truth for world state
  and is managed by dedicated systems.

## Current State

The current ECS prototype contains:

- Components in `src/components/mod.rs`
- Resources in `src/resources/mod.rs` and submodules
- Systems in `src/systems/mod.rs` and separate system files
- UI systems in `src/ui/mod.rs`

Current state transitions use Bevy `States`.

## Required States

`GameState`:

- `Loading`
- `MainMenu`
- `Playing`
- `Paused`
- `GameOver`
- `Victory`

`EditorMode` is **not** a separate `GameState`. It is a resource that
pauses normal gameplay systems while staying inside `Playing`.

## Core Entities

Expected high-level entity types:

### Player / Drone Entities

- `Player`
- `Blackbox`
- `Drone { index }`
- `Controllable`
- `Velocity`
- `Rotation`
- `AngularVelocity`
- `CircleCollider`
- `PhysicsBody`
- `ActiveViewUpgrade`
- `LightReceiver` if a visible-light camera is installed
- optional ray instrument components such as `VisibleCameraView`, `LidarView`,
  `ThermalView`, `RadarView`

### Ship / Room / Graph Entities

- `Ship`
- `RoomNodeEntity` (entity representing a room node in the typed graph)
- `TileNodeEntity` (entity representing a tile node)
- `RoomNode` component (data referencing graph node ID)
- `TileNode` component
- `RoomEdge` component for typed edges (e.g., electricity, fluid, door)
- `DockingPort`
- `RoomControl` indicating faction ownership

### Sensor / Light Entities

- `LightEmitter`
- `LightReceiver`
- `SensorEmitter`
- `SensorGridCell` (local cell state)
- `SensorGrid` resource manages the global grid

### UI Entities

- `LoadingScreen`
- `MainMenuScreen`
- `PauseScreen`
- `GameOverScreen`
- `HudScreen`
- `EditorScreen`
- `CommandLineScreen`
- `SchematicScreen`
- `ConfigScreen`

## Core Resources

### Already Present

- `PhysicsConfig`
- `InputConfig`
- `EditorMode`
- `EditorTool`
- `DiegeticView`
- `SchematicLayer`
- `SchematicLayers`
- `FocusedEntity`
- `ShipTemplateLibrary`
- `ActiveScenario`
- `ProcGenRegistry`
- `LoadingTimer`

### To Add or Formalize

- `RoomGraph` (typed room/tile graph)
- `SystemGraphs` (electrical, fluidics, atmospherics, thermal, computer)
- `SensorGrid` (shared cellular automaton grid)
- `LightGrid`
- `VisibilityCache`
- `ResearchState`
- `UpgradeRegistry`
- `PhysicsBroadphase`
- `ArenaEventLog` for automated testing

## Scheduling

### State-Specific Systems

- `Loading`
  - `spawn_loading_screen`
  - `loading_screen_tick`

- `MainMenu`
  - `spawn_main_menu`
  - `despawn_main_menu`
  - `main_menu_input`
  - `main_menu_button_system`

- `Playing`
  - gameplay systems run under `playing_and_not_editor`
  - editor systems run when `EditorMode.active`
  - UI overlay visibility systems run for command line, schematic map, config
    view
  - drone focus input runs in both normal play and editor mode
  - graph and sensor systems run at fixed frequencies

- `Paused`
  - `spawn_pause_menu`
  - `despawn_pause_menu`
  - `pause_menu_input`
  - `pause_menu_button_system`

- `GameOver`
  - `spawn_game_over_screen`
  - `despawn_game_over_screen`
  - `game_over_input`
  - `game_over_button_system`

### System Sets

The following logical system sets are recommended:

- `InputSet`
- `MovementSet`
- `PhysicsSet`
- `SensorSet`
- `VisibilitySet`
- `FsaSet`
- `GraphUpdateSet`
- `AISet`
- `ShipSystemSet`
- `ResearchSet`
- `UIOverlaySet`
- `CameraSet`

Bevy ordering should be explicit inside each set and between sets when
dependencies exist.

## Component / Resource Ownership Rules

- Components are data only.
- Systems should mutate only the components they explicitly query.
- Resources should represent singletons like configs, registries, and caches.
- Events should be preferred for decoupled notifications (collision events,
  damage events, research progress events, UI events).
- The graph resource `RoomGraph` should only be mutated by graph update
  systems, not by UI or input systems.

## UI Separation

The world camera feed and UI overlays must remain architecturally separate.

- The world camera is a 3D camera with order `0`.
- The UI camera is a 2D camera with order `10` and `IsDefaultUiCamera`.
- The schematic map is UI only.
- The command line is UI only.
- The config view is UI only.
- The focused view is an abstraction that may use the world camera or custom
  rendering, but must not be confused with UI overlays.

## Focused View and View Upgrades

The current `DiegeticView` enum includes:

- `FirstPerson`
- `CommandLine`
- `Schematic`
- `ConfigView`

`FirstPerson` currently means the visible-light camera view. It will later
become one of several focused view modes.

Planned view upgrade components:

- `VisibleCameraView`
- `LidarView`
- `ThermalView`
- `RadarView`
- `ActiveViewUpgrade`

The focused view system should query the focused entity, find
`ActiveViewUpgrade`, and render the corresponding view. If no view upgrade
is active, the player pilots blind.

## Drone and Fleet Configuration

- `1`-`9` focus individual drones.
- `0` opens the fleet configuration view, not a drone view.
- The Blackbox does not have its own camera perspective unless carried by
  another drone.

The config view should display:

- Blackbox status and upgrades
- drone status, index, and upgrades
- inventory / loadout information
- no live sensor data

## ECS Implementation Steps

### Step 1: Formalize component files

Move physics-related components into `src/components/physics.rs`.
Keep generic components in `mod.rs` re-exported for convenience.

### Step 2: Formalize system sets

Add Bevy `SystemSet` definitions to reduce ambiguous ordering.

### Step 3: Add graph components and resources

Add `RoomNode`, `TileNode`, `RoomEdge`, `SystemFlags`, `RoomGraph`,
`SystemGraphs`.

### Step 4: Add sensor grid and light resource

Add `SensorGrid`, `LightGrid`, `VisibilityCache`.

### Step 5: Split gameplay systems

Move:

- input mapping to `input.rs`
- movement integration to `movement.rs`
- physics resolution to `physics.rs`
- camera follow/focused view to `camera.rs`
- graph update to `graph_update.rs`
- sensor propagation to `sensor_propagation.rs`
- FSA updates to `fsa_update.rs`

Keep `systems/mod.rs` as the public export module.

### Step 6: Introduce events

Add:

- `CollisionEvent`
- `DamageEvent`
- `ResearchProgressEvent`
- `SensorPingEvent`
- `ViewUpgradeChangedEvent`
- `GraphEdgeStateChangedEvent`
- `FsaTransitionEvent`

### Step 7: Introduce shared registry resources

Use `ProcGenRegistry` for generation, `SensorRegistry` for sensors,
`UpgradeRegistry` for upgrade-driven actor assembly, and later
`ResearchState`.

## BDD Scenarios

### Scenario: Game starts in Loading then MainMenu

- Given the app starts
- Then it enters `Loading`
- Then after the loading timer completes it enters `MainMenu`
- And the main menu UI is visible

### Scenario: Starting the game enters Playing and spawns world

- Given the game is in `MainMenu`
- When the player selects `Start Game`
- Then the game enters `Playing`
- And `setup_scene` spawns the default ship scaffold and player/drone entities
- And the HUD appears
- And the typed room/tile graph is initialized

### Scenario: Pressing Escape opens pause menu

- Given the game is in `Playing`
- And no diegetic overlay is open
- When the player presses `Escape`
- Then the game enters `Paused`
- And the pause menu is shown

### Scenario: Entering editor from pause menu

- Given the game is in `Paused`
- When the player selects `Enter Editor`
- Then `EditorMode.active` is set to `true`
- And the game returns to `Playing`
- And normal gameplay systems are suspended by `playing_and_not_editor`
- And the editor UI appears

### Scenario: Pressing 0 opens config view

- Given the game is in `Playing`
- When the player presses `0`
- Then `DiegeticView` changes to `ConfigView`
- And the config view UI spawns
- And no entity focus changes
- And no sensor feed is shown

## Automated Arena Testing and Deterministic ECS

This section describes how the ECS should support automated playtesting.

### Headless Deterministic Mode

Bevy should support a headless mode with:

- no rendering
- no audio
- no UI interaction
- fixed timestep
- seeded random number generation
- deterministic world loading
- fast-forward simulation

This allows thousands of short gameplay episodes to be run automatically.

### Event Logging

Add a structured event log resource or event channel that records gameplay
events during arena episodes.

Examples:

- `MovementEvent`
- `SensorDetectionEvent`
- `GraphEdgeStateChangeEvent`
- `SalvageActionEvent`
- `SystemFailureEvent`
- `CascadeEvent`
- `SoftlockEvent`
- `RayInstrumentChangeEvent`
- `FsaTransitionEvent`

These events are the basis for behavioral descriptors used by the search.

### Behavior Descriptors

After each episode, compute descriptors from the event log:

- active behavior state frequency
- number of distinct rooms entered
- ray instrument usage frequency
- sensor usage frequency
- movement style metrics
- combat avoidance/engagement
- resource extraction rate
- cascade count
- time spent blind
- number of failed actions
- FSA transition frequencies

These descriptors should be independent of ECS implementation details.

### Upgrade Registry Requirement

The ECS must include a central `UpgradeRegistry` that maps upgrade IDs to
component constructors. Both player drones and generated NPCs are assembled
from this registry.

This is necessary for automated testing to sample arbitrary upgrade sets
without hardcoding archetypes.

### Resource Ownership

The arena runner must be able to reset the world between episodes. Therefore:

- `GameplayEntity` must be used consistently.
- world despawn/respawn must be quick and leak-free.
- `Commands` should be applied deterministically.
- The `RoomGraph` and `SensorGrid` must be rebuilt cleanly for each episode.

### BDD Scenario for Arena Testing

#### Scenario: Deterministic arena episode replays identically

- Given two arena runs with the same seed and same configuration hash
- When the simulation runs for a fixed number of ticks
- Then all entity states, event logs, and behavior descriptors match exactly
- And the typed graph and sensor grid states match

#### Scenario: Sampled drone build is symmetrical to NPC build

- Given a controlled drone with upgrade set U in arena A
- And an NPC with upgrade set U in arena B
- When both are driven by the same external objective
- Then their movement and behavior descriptors should be statistically similar

#### Scenario: Softlock is detectable

- Given an arena episode
- When the agent remains in the same small area without progress for N ticks
- Then a `SoftlockEvent` is emitted
- And the episode is flagged as invalid for search scoring

These scenarios ensure that automated playtesting is not corrupted by
non-deterministic behavior or hidden state.
