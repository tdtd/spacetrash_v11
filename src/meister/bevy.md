# Bevy Runtime Architecture

Bevy is the authoritative real-time simulation layer for **Spacetrash v11**. Eldiron may author ship templates, scenario data, and other content, but once play begins, Bevy owns the world, the ECS state, the renderer, and the gameplay loop.

## Current Role

Bevy is not used as a generic UI toolkit or as a script host. It provides:

- entity-component-system storage
- scheduling and state management
- rendering
- input
- UI layout
- asset/mesh/material creation for the prototype
- the extension point for first-person cameras, drone focus, and diegetic interfaces
- the runtime platform for the typed room/tile graph, FSA, sensor grid, and headless deterministic simulation

The game currently targets Bevy 0.13 with `DefaultPlugins`.

## ECS Separation

The project follows a strict ECS split:

- **Components** live in `src/components/mod.rs`
- **Resources** live in `src/resources/mod.rs` and its submodules
- **Systems** live in `src/systems/mod.rs` and separate system files
- **UI systems** live in `src/ui/mod.rs` and per-screen modules

Authoring data is not Bevy ECS data. It is deserialized into plain Rust types first, then converted into entities and components only when the world is spawned.

The typed room/tile graph is stored in a `RoomGraph` resource, not as loose entities. System graphs are derived from it.

## Scheduling

Bevy's `States` feature drives the high-level app flow.

The current `GameState` variants are:

- `Loading`
- `MainMenu`
- `Playing`
- `Paused`
- `GameOver`
- `Victory`

Systems are scoped with:

- `OnEnter`
- `OnExit`
- `Update`
- `.run_if(in_state(...))`
- custom condition helpers such as `playing_and_not_editor`

This keeps menu logic from running during gameplay and prevents gameplay systems from running in editor mode.

Graph update, sensor propagation, and FSA updates are scheduled at fixed
frequencies (e.g., 1 Hz for ship systems, per tick for physics, and per
sensor update interval).

## Camera Setup

The prototype creates two separate cameras at startup:

- a 3D world camera with order `0`
- a 2D UI camera with order `10`, marked as `IsDefaultUiCamera`

This split is important because the schematic map, command line, and editor UI are UI overlays. They must never reposition or rotate the 3D world camera.

Future work will add an `ActiveWorldCamera` marker so first-person drone and Blackbox views can be swapped without querying every camera.

## Input

Input is read through Bevy's `ButtonInput<KeyCode>`.

Reserved gameplay bindings are implemented in `src/systems/mod.rs`:

- `WASD` / arrow keys move the focused entity
- `1` through `9` focus drones
- `0` focuses the Blackbox
- backtick toggles command line / schematic map
- `F1` through `F12` switch schematic layers
- `Escape` opens the pause menu
- `K` is a debug game-over shortcut

Editor mode is deliberately entered and exited through buttons. The prototype no longer maps function keys or CapsLock to editor mode.

## UI

Bevy UI is used for:

- loading screen
- main menu
- pause menu
- game-over screen
- HUD
- editor toolbar and status bar

UI entities are ordinary Bevy entities with `NodeBundle`, `TextBundle`, `ButtonBundle`, and marker components. Systems query for `Interaction` and marker components to handle buttons.

The editor overlay is spawned and despawned by watching `EditorMode`. Toolbar buttons update `EditorTool`.

The HUD and editor overlay display the current `DiegeticView` and `SchematicLayer`.

## Ship Generation and World Spawning

World creation starts from `setup_scene`. That system reads a local `ShipTemplateLibrary`, selects the default `ShipTemplate`, and calls `spawn_ship_from_template`.

The generator creates the planar room scaffold, then runs WFC interior stub, then builds the typed room/tile graph, and finally spawns Bevy entities for:

- ship entities
- room nodes
- tile nodes (if needed)
- typed edges (as components)
- wall segments with colliders
- docking ports

Walls use `AabbCollider` entities. Doors are represented as gaps, not as separate door entities in the current prototype.

## Arcade Physics

Physics is custom, not a third-party physics engine. Bevy supplies the math primitives and schedules, but movement and collision resolution are implemented in systems.

Current systems:

- `player_input_system`
- `movement_system`

These update `Velocity` and `Transform` and resolve overlap against static `AabbCollider` entities.

Future physics will add:

- `CircleCollider`
- `AngularVelocity`
- `Rotation`
- actor sizes
- fixed timestep integration
- collision events

## First-Person Drone View

Bevy cameras will be repositioned from focused entity transforms. A focused entity may carry a `FirstPersonCamera` component with an eye offset.

The world camera will alternate between:

- top-down prototype view
- first-person drone view

Visible-light requirements will be modeled later with `LightEmitter`/`LightReceiver` and the sensor grid, but the prototype currently uses a global directional light.

## UI vs World Rendering

The schematic map is not a camera. It is a UI conceptual layer.

This keeps the render pipeline predictable:

- 3D world camera renders the continuous world view
- 2D UI camera renders all overlays
- diegetic overlays never manipulate world camera data

## Performance

The prototype is intentionally small.

Expected budget:

- 20 active dynamic actors
- 50 rooms
- one continuous world camera
- one UI camera

Bevy's multithreaded schedule is available, but the current systems remain lightweight enough for prototype use.

## Future Bevy Usage

Bevy is the long-term runtime for:

- first-person drone and Blackbox cameras
- dynamic docking graph rendering
- live fog-of-war and sensor-driven visibility
- hidden research system UI
- fluidics and atmospherics simulation outputs
- modular actor upgrade spawning
- ship system failure visualization
- typed room/tile graph editing in god mode

Bevy remains the runtime; Eldiron remains the authoring layer.

## Automated Headless Testing Support

For automated playtesting, Bevy must support a headless mode.

### Required Headless Properties

- no rendering
- no audio
- no UI event loop
- fixed timestep
- seeded random generation
- deterministic world loading
- ability to fast-forward simulation

This mode should be selectable via a command-line flag or compile feature:
`--headless` or `cfg(feature = "headless")`.

### Deterministic Schedule

Systems must run in a fixed, reproducible order. Avoid unordered iteration
over entities where possible; use explicit `Query` ordering and stable
containers.

### Event Logging

Add a central event log resource or channel that records:

- entity spawn/despawn
- movement
- sensor grid propagation
- door state changes
- graph edge state changes
- salvage actions
- system failures
- cascade events
- softlocks
- ray instrument changes
- FSA transitions

This log feeds behavior descriptors for automated search.

### Arena Runner

The arena runner is a separate Bevy `App` that loads a generated scenario and
a sampled drone/NPC configuration, then runs for a fixed number of ticks
without user input.

Key responsibilities:

- reset world between episodes
- apply deterministic seeds
- record behavior descriptors
- emit softlock events if no progress

The runner should not be tied to the normal game UI or main menu flow.

### BDD Scenario for Headless Mode

#### Scenario: Headless run matches graphical state

- Given a fixed scenario and input sequence
- When the game runs in both graphical and headless modes
- Then at tick N, entity transforms, velocities, and sensor states must match
- And the headless event log is a prefix of the graphical event log after
  ignoring UI/rendering events
- And the typed graph and sensor grid states match

This ensures the automated testing reflects real gameplay.
