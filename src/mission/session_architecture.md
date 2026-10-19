# Session Architecture Insights

This document captures the significant architectural insights and plans
discussed during this session. It is intended to sit alongside the other
focused mission plans and design documents.

The session explored several overlapping ideas:

- automated arena playtesting
- upgrade-driven actor generation
- single ray-instrument sensors
- cellular-automaton raycasting and sensor grids
- light emitters and receivers
- Factorio-style signal propagation
- planar room scaffold + WFC interior growth
- room types as progression gates for ship systems
- typed tile/room graph representation
- finite-state automata for runtime systems

These ideas are not independent. Together they form a unified approach for
procedural generation, runtime simulation, UI/diegetic interface, god mode,
and automated playtesting.

## 1. Greatest Technical Risk

The greatest risk is trying to create too many deep, interacting systems
before a small coherent vertical slice is stable.

Specific risk areas are:

### Cascade instability

Ship systems such as electrical, fluidics, atmospherics, termals, and
computer can cascade when salvaged or damaged. This is desirable, but
tuning it is difficult. Small parameter changes can produce degenerate
chain reactions, instant kills, or zero consequences.

### Upgrade-driven actor complexity

Actors are assembled from upgrade modules, including behavior upgrades.
This is powerful, but dynamically composing ECS entities from authored
data can become brittle. Procedural enemies can produce overpowered or
useless combinations if not carefully validated.

### Ray/visibility complexity

If implemented as full per-frame raycasts, sensor and camera views will
become expensive. The single ray-instrument constraint is essential, and
the chosen representation will determine whether visibility remains
tractable.

### Generation / docking / editor interaction

Deterministic procedural ship generation, WFC interiors, docking,
save/load, and an editor are individually manageable, but integrating them
is risky.

The safe path is to build and validate these systems separately, then
integrate them through a shared tile/graph representation.

## 2. Automated Arena Playtesting: Upgrade-Driven Search

Automated playtesting should not rely on handcrafted agent archetypes
such as “fast explorer” or “cautious salvager”.

The core principle is:

- Every actor is an instance of the same upgrade system.
- Player drones and generated NPCs are assembled from the same upgrade pool.
- Behavior is an emergent property of installed upgrades, not a predefined
  species or class.
- A test drone is simply a drone entity with a sampled upgrade set.

This means testing should build one actor assembler and evaluate sampled
upgrade configurations inside generated arenas.

### Separation of systems

The following systems should be implemented and tested separately:

1. Physics
2. Ray / visibility
3. Level generation and editing
4. Upgrade definition and actor assembly
5. Behavior arbitration and execution
6. Ship system simulation
7. Arena runner
8. Search and evaluation

Only then should they be combined in the arena.

### Headless deterministic episodes

Every arena episode should be:

- headless
- deterministic from a seed
- fixed timestep
- event logged
- fast-forwardable

### Behavioral descriptors

Do not evaluate only win rate. Instead, collect descriptors such as:

- active behavior state frequency
- sensor usage frequency
- number of distinct rooms entered
- movement style
- combat avoidance vs engagement
- resource extraction rate
- cascade event count
- time spent blind
- ray-instrument usage
- softlock count

These descriptors should be computed from a structured event log.

### MAP-Elites

MAP-Elites, or another quality-diversity algorithm, is recommended.

Search dimensions:

- behavior upgrade combination
- movement upgrade
- active ray instrument or none
- weapon/effector
- chassis size
- sensor package
- level layout
- enemy distribution
- initial ship system state

Quality should reward:

- purposeful, non-degenerate movement
- meaningful response to threats and resources
- distinct behavior profiles
- recoverable cascades
- multiple viable tactics

Do not reward only winning.

### Validation before scoring

Reject configurations with:

- collider penetration
- NaN positions/velocities
- zero-cost sensors
- impossible reachability
- instant death
- permanent softlock

These are simulation failures, not interesting gameplay.

## 3. Upgrade System Underlies Everything

The upgrade system is not a player-only inventory system. It is the
fundamental actor generator for players, NPCs, monsters, and future crew.

An actor is:

- a chassis
- a set of upgrade modules
- runtime state generated from those modules

Upgrade categories include:

### Movement

- `WheelDriveUpgrade`
- `LegsUpgrade`
- `HoverUpgrade`
- `TracksUpgrade`
- `ThrusterUpgrade`

These map to physics body, collider size, and movement feel.

### Sensory

- `VisibleCameraUpgrade`
- `LidarUpgrade`
- `RadarUpgrade`
- `ThermalUpgrade`
- `MicrophoneUpgrade`
- `MotionDetectorUpgrade`

Only one sensory upgrade may be an active ray instrument at a time.

### Effector / weapon

- `TurretGunUpgrade`
- `MeleeArmUpgrade`
- `GrabberUpgrade`
- `RepairToolUpgrade`
- `LightUpgrade`
- `ExtinguisherUpgrade`
- `HackingModuleUpgrade`

### Defensive

- `ArmorPlateUpgrade`
- `EnergyShieldUpgrade`
- `FireproofPlatingUpgrade`
- `RadiationShieldingUpgrade`

### Utility

- `BatteryUpgrade`
- `CargoBayUpgrade`
- `ScannerUpgrade`
- `TractorBeamUpgrade`

### Behavior

- `PatrolBehavior`
- `GuardBehavior`
- `AggressiveBehavior`
- `FleeBehavior`
- `ScavengeBehavior`
- `FollowLeaderBehavior`
- `HuntBehavior`
- `ProtectBehavior`

Behavior upgrades are components. Multiple behavior components may exist,
and a behavior arbitration system selects the active one based on priority
and current sensor state.

## 4. Single Ray-Instrument Constraint

Every actor is allowed at most one active ray-based instrument.

Examples:

- visible-light 3D camera
- LIDAR
- turret gun using ray-based aiming
- thermal imager
- radar emitter/receiver

The visible-light camera is not a separate two-ray system. It is simply a
light-ray-detecting 3D camera. It detects visible light arriving along
straight lines through the scene.

This constraint is a performance baseline and a gameplay choice generator.

Non-ray upgrades such as lights, microphones, and movement modules may be
installed alongside the single ray instrument.

## 5. Cellular-Automaton Raycasting

Rather than dense per-agent raycasting, the world should use a shared
tile/room graph sensor state and propagate sensor knowledge as local
automata.

Precedent includes:

- roguelike field-of-view algorithms
- Minecraft-style block light propagation
- grid flood fill / Dijkstra visibility

For Spacetrash:

### Visible-light camera

- Runs as a bounded frustum/cone propagation from the camera cell.
- Requires visible light.
- Dark cells are not marked visible.

### LIDAR

- Uses structural wavefront propagation.
- No light required.
- Returns coarse structural geometry, not textures or identities.

### Thermal

- Reads local heat values from a shared thermal grid.
- Does not require visible light.
- May be blocked by solid/insulated cells.

### Radar

- Runs as a slow pulse.
- Between pings, information remains stale.
- Detects moving objects but not static detail.

The shared grid allows many receivers to reuse one propagation pass,
which is the core performance advantage.

## 6. Light Emitters and Receivers

A clean ECS abstraction is:

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

- `LightEmitter` seeds a local light value.
- A propagation system spreads that value through the tile graph.
- `LightReceiver` samples light at its current cell.

This cleanly separates:

- a camera is a `LightReceiver`
- a drone headlamp is a `LightEmitter`
- a light source is an effector, not a sensor
- the one-ray constraint applies to receivers, not to emitters

## 7. Factorio-Style Signal Networks

Factorio’s circuit network is a strong model for Spacetrash sensors.

Instead of continuous rays, sensors become signal endpoints over a
connected graph.

Example signal types:

- `VisibleLight`
- `Thermal`
- `RadarPing`
- `LidarEcho`
- `StructuralEcho`
- `Power`
- `Data`

The world graph already has edges for:

- structural adjacency
- doors
- vents
- pipes
- wires
- computer network

These edges can carry signal types. Wall/opaque tiles block relevant
signals; doors and windows may pass only certain spectra.

This makes sensors cheap, deterministic, and easy to test.

## 8. Planar Room Scaffold + WFC Interior Growth

The most important generation insight is:

> Do not use WFC to generate the whole ship.  
> Generate a planar room graph first, then use WFC only to grow local
> interiors onto that scaffold.

### Scaffold

The scaffold is a planar graph:

```text
RoomNode {
    id,
    room_type,
    shape,
    system_flags,
}

RoomEdge {
    kind,
    from_room,
    to_room,
    location,
}
```

Edge kinds include:

- `Door`
- `Corridor`
- `Airlock`
- `Umbilical`

The scaffold can be:

- manually authored in Eldiron
- procedurally generated
- built by docking multiple ships

### WFC interiors

Each room graph node is expanded into a local tile grid.

WFC uses room-type-specific tile sets:

- `Storage`: clutter, containers
- `Reactor`: reactor, coolant inlet, power conduit
- `LifeSupport`: vents, scrubbers, oxygen tanks
- `ComputerCore`: terminals, routers, camera nodes

WFC preserves:

- room boundary
- door positions from `RoomEdge`
- system ports required by adjacent rooms
- local interior connectivity

WFC no longer has to solve global system connectivity. That is handled by
the scaffold graph and system graph generation.

## 9. Room Types as Progression Gates

The tech tree is less about unlocking individual upgrades and more about
introducing systemic layers through room types.

Examples:

- `Engineering` introduces basic electrical management.
- `Reactor` introduces thermal management.
- `LifeSupport` introduces atmospherics.
- `Hydroponics` introduces fluidics.
- `ComputerCore` expands computer/network and sensors.
- `Bridge` exposes system maps and ship-wide diagnostics.

Early ships may have no fluidics, no atmospherics, and no thermal systems.

When the player encounters a ship with the associated room, that system
becomes locally relevant.

Advantages:

- complexity is introduced gradually through exploration
- system simulation is parceled out by room type
- each room becomes a gameplay mechanic gate
- docking compatibility can compare system flags

## 10. Typed Tile/Room Graph Representation

The world should be represented as a graph of nodes and typed edges.

Nodes are tiles/rooms.

Edges types:

- structural adjacency
- door
- power wire
- fluid pipe
- ventilation duct
- computer network

Each node stores:

- structural kind
- physics/occupancy state
- electrical state
- fluidics state
- thermal state
- computer state
- fog-of-war state

Each edge may also be an FSM object:

```rust
struct Edge {
    kind: EdgeKind,
    from: NodeId,
    to: NodeId,
    capacity: f32,
    state: EdgeState,
}
```

System graphs are derived from scaffold edges and room system flags.

This graph becomes the single source of truth for generation, simulation,
saving, loading, and god mode.

## 11. FSA Runtime

Finite-state automata are a good fit for Spacetrash.

Use FSAs for:

- sensors / ray instrument update modes
- behavior arbitration
- door state
- electrical state
- fluidics state
- thermal state
- atmospherics state
- fog-of-war cell state
- test episode progression

FSAs should be local and small. Avoid one giant global FSA.

Transition tables can be authored in Eldiron later.

This makes runtime state explicit, deterministic, and replay-friendly.

## 12. Scale Estimates

The originally stated prototype target is:

- 20 dynamic actors
- 50 rooms
- one focused view per player

This is appropriate for the first vertical slice, but too low for the
final game.

Staged targets:

### Prototype

- 20 actors
- 50 rooms
- 1 focused view
- 1–3 docked ships

### Mid-development

- 50 actors
- 100 rooms
- scheduled ray instruments and tile sensors
- 2–5 docked ships

### Later / content-complete

- 80–120 actors
- 200+ rooms
- fleet of 9 + enemies + relays
- multiple docked ships
- low-frequency ship-system simulation across all rooms

The cellular/tile/FSA approach should make these later numbers feasible
without requiring a full navmesh or continuous raycasting simulation.

## 13. Recommended Immediate Path

1. Build the planar room scaffold and typed graph representation.
2. Add a basic WFC interior pass that preserves room doors and ports.
3. Implement a shared tile sensor grid.
4. Add `LightEmitter` and `LightReceiver`.
5. Implement the single active ray-instrument per actor.
6. Add first behavior FSA + arbitration.
7. Build headless deterministic arena runner.
8. Add structured event logging and behavior descriptors.
9. Map generated actors/enemy profiles from the same upgrade registry.
10. Run shallow MAP-Elites searches on small scenarios.

Do not attempt full cascading ship systems until the scaffold/graph/sensor
core is stable.

## 14. Summary

The session produced a coherent target architecture:

- planar room graphs define the gameplay skeleton
- WFC fills local room interiors
- typed graph edges connect rooms, doors, pipes, wires, vents, networks
- FSA operates on nodes and edges
- cellular-automaton propagation replaces most expensive raycasting
- `LightEmitter` / `LightReceiver` model light and sensors
- single ray-instrument constraint bounds per-agent cost
- upgrade-driven actors generate both player drones and procedural enemies
- automated MAP-Elites search evaluates behavior, not win rate
- room types act as progression gates for ship systems

This combination aligns generation, runtime, editing, sensors, AI, and
automated testing into a single deterministic tile/graph framework.
