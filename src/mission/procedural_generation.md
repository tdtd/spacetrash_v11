# Procedural Generation Mission Plan

## Purpose

This document defines the implementation plan for procedural generation
using a **planar room scaffold** and **Wave Function Collapse (WFC) interior
growth**, with room types acting as progression gates for ship systems.

## Goals

- Generate the ship skeleton as a planar graph of rectangular rooms.
- Use WFC only to populate local room interiors on that scaffold.
- Connect rooms with 1-tile-wide doors and 1-tile deadzones.
- Encode room types and system flags to control which ship systems exist.
- Support manual overrides from Eldiron-authored templates.
- Prepare for deterministic, automated generation search.

## Current State

The prototype has:

- `ShipTemplate` struct with seed, room count, size ranges, door width,
  wall thickness, wall height, grid dimensions, corridor width, manual
  room overrides, door overrides, and docking ports.
- `default_ship_template()` returning a 3x2 room grid with 1-tile doors and
  corridors.
- `spawn_ship_from_template()` that builds a grid of rectangular rooms and
  wall segments.
- `ShipGenerationAlgorithm` trait and `ProcGenRegistry` resource for future
  pluggable algorithms.

The current generator uses a simple deterministic grid arrangement. It is
not randomized yet, but it serves as the baseline.

## Required Data Structures

### Room Scaffold

The scaffold is a planar graph:

```rust
struct RoomNode {
    id: RoomId,
    room_type: RoomType,
    shape: Rect,
    system_flags: SystemFlags,
}

struct RoomEdge {
    kind: EdgeKind,
    from: RoomId,
    to: RoomId,
    location: EdgePlacement,
}
```

`RoomType` and `SystemFlags` are essential for progression.

### System Flags

```rust
bitflags! {
    struct SystemFlags: u8 {
        const ELECTRICAL = 1 << 0;
        const THERMAL    = 1 << 1;
        const ATMOSPHERICS = 1 << 2;
        const FLUIDICS   = 1 << 3;
        const COMPUTER   = 1 << 4;
    }
}
```

### WFC Tile Sets

Each room type has a WFC tile set and adjacency constraints.

### Scenario

Stores ship instances and docking links.

### ProcGen

- `ProcGenContext`
- `ShipGenOutput`
- `ShipGenerationAlgorithm`
- `ProcGenRegistry`

## Generation Pipeline

1. **Scaffold generation**  
   Produce a planar graph of room nodes and edges. This can be authored
   in Eldiron, generated procedurally, or assembled by docking ships.

2. **WFC interior generation**  
   For each room node, expand its rectangle into a local tile grid and run
   WFC using the room-type-specific tile set. Preserve door positions and
   system ports.

3. **System graph construction**  
   From scaffold edges and interior tile data, create typed graphs for
   electrical, fluidics, atmospheric, thermal, and computer systems.

4. **Runtime entity spawning**  
   Convert the generated graph into Bevy entities/components.

## Room Types and Progression

Room types act as progression gates:

| Room Type        | System Flags          | Introduces             |
|------------------|-----------------------|------------------------|
| `Storage`        | none                  | none                   |
| `Engineering`    | ELECTRICAL            | basic power            |
| `Reactor`        | ELECTRICAL, THERMAL   | power + heat           |
| `LifeSupport`    | ATMOSPHERICS          | air / pressure         |
| `Hydroponics`    | FLUIDICS, ATMOSPHERICS| fluids + air           |
| `ComputerCore`   | COMPUTER              | network / sensors      |
| `Bridge`         | COMPUTER, ELECTRICAL  | ship-wide system maps  |

Early ships may have only `Engineering`. Complex ships include multiple
room types, introducing systems gradually.

## WFC Interior Rules

- WFC operates only inside room boundaries.
- Room connectivity and door placement are unaffected by interior details.
- Interior tiles may include diagonal walls, low cover, consoles, and props.
- Diagonal occluders are limited to the room interior.
- Door tiles align to the 1-tile door openings on room boundaries.
- Tile sets are selected by room type.

## Docking: Hybrid Space Graph and Shared Interior Map

Spacetrash uses a hybrid docking model:

- Space navigation is an abstract graph; no 3D ship flight physics.
- After docking, ships and station merge into one shared interior `RoomGraph` and tile grid.
- Docking itself is an FSA + deterministic merge/undock transaction.

This departure from Duskers is the core new mechanic. It enables boarding, shared maps, system transfer, and multi-ship stations.

### Space Navigation Layer

Model the fleet/station world as a directed/undirected graph of ship and station nodes. Docking links are edges. Travel between distant objects is a command: dock, undock, launch shuttle, etc.

This layer is FTL-like and keeps ship management simple.

### Interior Merged Map

After docking, the involved instances become a single passable playfield:

- merged `RoomGraph`
- merged tile grid
- generated umbilical corridor
- docking-port room nodes
- connected colliders
- derived `CurrentSystemGraphs`

The existing `merge_instances` function in `src/procgen/composite.rs` is the Phase 1 implementation.

### Docking FSA and Transactions

Docking is a state machine:

`Separated -> Aligned -> Clamped -> Sealed -> Powered -> Pressurized -> Open`

Failure states include `Rejected`, `Misaligned`, `Jammed`, `Unpowered`, `Breached`, `Decompressing`.

Dock and undock are implemented as deterministic world transactions:

**Dock:**

1. Validate airlock compatibility and distance.
2. Generate or load local ship/station grids.
3. Generate a two-tile-wide umbilical corridor.
4. Merge tile grids without overlap.
5. Merge `RoomGraph`s with unique room IDs.
6. Add `RoomEdge` with `EdgeKind::Umbilical`.
7. Rebuild affected tile/room/collider entities.
8. Rebuild `CurrentRoomGraph` and `CurrentSystemGraphs`.
9. Set docking FSA to `Open`.

**Undock:**

1. Set FSA to `Sealed`, `Breached`, or `Decompressing`.
2. Remove umbilical corridor tiles.
3. Split room graph at the dock edge.
4. Despawn only affected corridor/collider entities.
5. Keep local instance data so re-docking is possible.

### Umbilical Corridor

The umbilical is a two-tile-wide passable segment. It is generated as real tile/room geometry, not only a graph edge. It connects two main airlock rooms and carries only systems supported by both sides.

### Four-Sided Station

The station is a central hub with one airlock per side. Four ships dock at cardinal offsets. The resulting merged graph is a single connected playfield.

The station introduces:

- dock control room
- station-wide system maps
- central chokepoints
- airlock lockdown state

### UI / Editor Integration

The editor must support:

- placing ship/station instances
- moving and rotating them
- snapping compatible airlocks when within max umbilical length
- generating/removing umbilical corridors
- showing docking state and system compatibility

Terminal/HUD commands should include docking actions:

- `open <airlock-id>`
- `close <airlock-id>`
- `dock <port-id>`
- `undock <port-id>`
- `status dock`

Schematic layers must visualize docking edges, umbilical routes, and locked/open states.

### Testing and Map Contracts

Add tests for:

- merged room graph connected
- no tile overlap from different instances
- each umbilical is exactly two passable tiles wide
- system graph pruning across umbilical edges
- every main airlock is either connected or intentionally empty
- four-sided station has four airlocks
- no orphan airlocks
- safe undock does not corrupt local instance data

Map assertions should be extended with composite-level checks.

### Implementation Steps

1. Add `DockingPlan` and composite spawn path.
2. Build a hardcoded four-sided station composite fixture.
3. Generate real umbilical tiles, not just graph edges.
4. Add docking FSA component/resource.
5. Implement deterministic dock/undock transactions.
6. Integrate `Scenario`, `ShipInstanceInit`, and `DockingLink` into world spawn.
7. Add editor docking tools and snap validation.
8. Add docking terminal commands and schematic overlays.
9. Add map contracts and BDD tests for docking.
10. Evolve `merge_instances` into a runtime-safe world transaction.

## Implementation Steps

### Step 1: Define scaffold types

Add `RoomNode`, `RoomEdge`, `RoomType`, `SystemFlags`.

### Step 2: Update `ShipTemplate`

Include scaffold data or a generation algorithm ID that produces a scaffold.

### Step 3: Build basic planar scaffold generator

Generate a connected planar graph of rooms with door edges.

### Step 4: Implement WFC interior stub

For each room, expand into a grid and fill with placeholder tiles,
preserving doors.

### Step 5: Derive system graphs

From scaffold edges and room flags, create typed system edges.

### Step 6: Spawn entities from scaffold + WFC

Convert the generated data into Bevy entities.

### Step 7: Add manual overrides

Allow Eldiron-authored manual rooms, door overrides, and system port
overrides.

### Step 8: Register default algorithm in `ProcGenRegistry`

Create `PlanarScaffoldAlgorithm` and register it.

### Step 9: Add deterministic world hashing

Every generated world should be reproducible from a configuration hash.

## BDD Scenarios

### Scenario: Default scaffold creates connected room graph

- Given a default ship template
- When the scaffold generator runs
- Then all rooms are connected by door edges
- And each room has a rectangular shape
- And the graph is planar

### Scenario: WFC preserves door positions

- Given a room with a door edge on its boundary
- When WFC fills the room interior
- Then the door tile remains open
- And no WFC tile blocks the door opening

### Scenario: Room types control system flags

- Given a scaffold with `Engineering`, `Storage`, and `LifeSupport` rooms
- When system graphs are derived
- Then only `Engineering` has electrical edges
- And only `LifeSupport` has atmospheric edges
- And `Storage` has no system edges

### Scenario: Docking connects only shared systems

- Given two ships, one with fluidics and one without
- When they dock
- Then the docking edge does not create a fluidics connection

## Automated Procedural Generation Search

Automated playtesting depends on generation being parameterized and
deterministic.

Search parameters:

- scaffold seed
- room type distribution
- room size ranges
- door placement
- WFC tile weights
- docking graph

Validation includes:

- no overlapping rooms
- no disconnected room graph
- no missing doors
- no unreachable docking ports
- all system graphs correctly derived

Quality metrics include:

- number of viable movement paths
- system complexity progression
- sensor-critical rooms
- cascade topology
- chokepoints without dead ends

MAP-Elites can evolve scaffolds and WFC weights.

## Immediate Next Steps

- Define scaffold types and system flags.
- Build basic planar scaffold generator.
- Add WFC interior stub.
- Derive system graphs.
- Connect to Bevy entity spawning.
- Add deterministic world hashing.

Do not implement full WFC interiors until the scaffold and door system are
stable.
