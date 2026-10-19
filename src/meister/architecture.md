# Spacetrash v11 Architecture

This document describes the high-level architecture for **Spacetrash v11**,
a 2D gameplay experience rendered with 3D graphics and powered by an
Entity-Component-System architecture.

## 1. Core Technology

- **Language:** Rust
- **ECS / Engine:** Bevy
- **Rendering:** Bevy’s 3D renderer, used with a top-down/isometric camera
  to present 2D gameplay with 3D visuals.
- **Physics:** Custom lightweight arcade physics layer, not tile/grid based.
- **World Structure:** Each ship is generated from a planar room scaffold.
  WFC fills room interiors on that scaffold. Ships dock to form a single
  dynamic playfield.
- **Rules / Content:** Eldiron authors room scaffolds, scenario data, and
  upgrade definitions. Bevy owns the real-time simulation.
- **Research / Tech Tree:** Room types act as progression gates for ship
  systems. Research modifies how well the player understands or manipulates
  those systems.
- **Inspiration:** Factorio for signal networks and emergent complexity,
  FTL for ship-system management and cascading failures.

The overall design emphasizes emergent gameplay from simple, interdependent
mechanics. The world is represented as a typed graph of room nodes and
system edges. Runtime state is managed by finite-state automata. Sensors
use a shared cellular-automaton sensor grid instead of dense raycasting.

## 2. World Representation

The core world model is a **typed room/tile graph**.

### Nodes

Nodes are rooms or tiles.

Room nodes contain:

- room type
- rectangular shape
- system flags
- system state (FSA)
- fog-of-war state

Tile nodes contain:

- structural kind
- physics/occupancy state
- electrical state
- fluidics state
- thermal state
- computer state
- fog state

### Edges

Edges are typed:

- `Structural` — physical adjacency
- `Door` — door state FSA
- `PowerWire`
- `FluidPipe`
- `Vent`
- `Network`
- `Airlock`
- `Umbilical`

Each edge may carry state and capacity.

This graph is the single source of truth for generation, simulation, saving,
loading, and god mode.

## 3. Game State and App Flow

The game uses a `GameState` enum with Bevy’s `States` feature:

- `Loading`
- `MainMenu`
- `Playing`
- `Paused`
- `GameOver`
- `Victory`

`EditorMode` is not a separate state. It is a resource that pauses normal
gameplay systems while staying inside `Playing`.

## 4. Arcade Physics System

Physics is custom, lightweight, and designed for top-down arcade gameplay.
It uses continuous movement on the XZ plane and is deterministic.

### Components

- `Velocity { linear: Vec2 }`
- `AngularVelocity { yaw: f32 }`
- `Rotation { yaw: f32 }`
- `PhysicsBody`
- `CircleCollider`
- `AabbCollider`
- `LineSegmentCollider`
- `ActorSize`

### Integration

Use semi-implicit Euler with fixed timestep (60 Hz). Damping, clamping,
and collision resolution are applied as described in `src/mission/physics.md`.

### Broadphase

Spatial hash grid with 2.0-unit cells. Static colliders are inserted at
generation time; dynamic actors are updated each frame.

## 5. Procedural Generation: Planar Scaffold + WFC

Ships are generated in two passes.

### Room Scaffold

A planar graph of room nodes and edges is generated or authored.

Room types include:

- `Storage`
- `Engineering`
- `Reactor`
- `LifeSupport`
- `Hydroponics`
- `Medical`
- `ComputerCore`
- `Command`
- `Bridge`
- `CargoHold`

Each room type carries `system_flags` indicating which ship systems are
present:

- `ELECTRICAL`
- `THERMAL`
- `ATMOSPHERICS`
- `FLUIDICS`
- `COMPUTER`

Early ships may have only `ELECTRICAL`. More complex ships introduce other
systems through specific room types.

### WFC Interiors

Each room scaffold node is expanded into a bounded local tile grid. WFC
fills that grid using room-type-specific tile sets.

WFC preserves:

- room boundary rectangle
- door positions from scaffold edges
- system ports required by adjacent rooms
- local connectivity

WFC does not need to solve global system connectivity. That is handled by
the scaffold and derived typed edges.

## 6. System Graphs and Progression

From the scaffold and interior tile data, higher-level system graphs are
created:

- electrical graph
- fluidics graph
- atmospherics graph
- thermal graph
- computer graph

These graphs only exist where the room/system flags permit.

### Progression Gates

The tech tree is expressed through room types:

| Room Type        | System Introduced      |
|------------------|------------------------|
| `Engineering`    | Basic electrical       |
| `Reactor`        | Thermal + high power   |
| `LifeSupport`    | Atmospherics           |
| `Hydroponics`    | Fluidics + atmosphere  |
| `ComputerCore`   | Computer/network       |
| `Bridge`         | Ship-wide system maps  |

The player discovers new systems by encountering and securing these rooms.
This parceling of complexity is a core design principle.

## 7. Runtime FSA

Finite-state automata govern runtime behavior.

### Node FSAs

- fog-of-war per tile
- thermal/fire per tile
- atmosphere per room node

### Edge FSAs

- door state per door edge
- valve state per fluid edge
- breaker/power state per electrical edge
- vent state per atmospheric edge

Transitions are data-driven and may be authored in Eldiron.

## 8. Sensors, Light, and Fog of War

Sensors do not perform dense per-frame raycasts. Instead they operate on a
shared **sensor grid** using cellular automaton propagation.

### Single Ray Instrument Constraint

Each agent has at most one active ray-based instrument.

Examples:

- visible-light camera
- LIDAR
- thermal imager
- radar
- turret gun using ray aiming

A visible camera is a `LightReceiver`. It requires a `LightEmitter`
somewhere in visible range.

### Light Emitters and Receivers

```rust
struct LightEmitter {
    kind: LightKind,
    spectrum: Spectrum,
    intensity: f32,
    range: f32,
}

struct LightReceiver {
    spectrum: Spectrum,
    sensitivity: f32,
    current_illumination: f32,
}
```

`LightEmitter` seeds the shared light grid. A propagation system spreads
light through transparent tiles. `LightReceiver` samples the local grid.

### Sensor Grid

The sensor grid is a tile-based structure storing:

- visibility state
- light intensity
- thermal value
- radar ping timestamp
- last sensor update

Sensors update only local regions and frequencies.

### Fog of War

Fog-of-war is a cellular automaton layer:

- `Unseen`
- `CurrentVisible`
- `RecentlyVisible`
- `Blocked`

Cells decay to `Unseen` if not updated by an active sensor.

## 9. Upgrade System and Actor Generation

Actors are assembled from upgrade modules. Player drones and NPCs use the
same upgrade pool.

### Chassis

- base collider size
- base movement profile
- base stats

### Upgrade Modules

Upgrade modules add components. Categories:

- movement
- sensory
- effector
- defensive
- utility
- behavior

Behavior upgrades contribute FSA states/transitions. A behavior arbitration
system selects the active behavior state.

See `src/mien/upgrades.md` and `src/mission/ai.md`.

## 10. Automated Arena Playtesting

After baseline systems are stable, an automated arena runner will evaluate
generated configurations.

The runner uses:

- headless Bevy mode
- deterministic fixed timestep
- seeded random generation
- structured event logging
- behavioral descriptors

Search uses MAP-Elites to find diverse, non-degenerate behavior. It does
not optimize only win rate.

## 11. Performance Considerations

- shared sensor grid amortizes visibility cost
- single ray instrument per agent caps per-actor raycasts
- FSA updates at low frequency (1 Hz for ship systems)
- spatial hash broadphase for physics
- deterministic headless mode for testing

Staged scale targets:

- Prototype: 20 actors, 50 rooms, 1–3 ships
- Mid: 50 actors, 100 rooms, 2–5 ships
- Later: 80–120 actors, 200+ rooms, multiple ships

## 12. Future Expansion

- Save/load system
- WFC interior content
- docking graph editor
- research system
- fluidics/atmosphere full simulation
- travel methods

## 13. Technical Roadmap

1. Implement planar room scaffold and typed graph.
2. Add basic WFC interior pass.
3. Implement shared tile sensor grid.
4. Add `LightEmitter` / `LightReceiver`.
5. Implement single ray instrument constraint.
6. Add behavior FSA + arbitration.
7. Build headless arena runner.
8. Add event logging and descriptors.
9. Run MAP-Elites searches on small scenarios.
10. Introduce room-type progression for ship systems.

This architecture aligns generation, runtime, editing, sensors, AI, and
automated testing into a single deterministic tile/graph framework.
