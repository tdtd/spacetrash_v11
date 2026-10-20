# Composing Multiple Ships into a Single Level

## Purpose

This document defines the implementation plan for composing multiple
ships into one continuous playfield. The most complex supported case is a
**four-sided space station**: four independent ships dock to a central
station that has one main airlock per side. Each ship connects to the
station through a stretchable umbilical between main airlocks.

The goal is to have a single, seamless world where all ships and the
station share the same room/tile graph and system graphs. The player can
move from one ship to another without any discrete level transitions.

---

## Terminology

- **Ship**  
  A independently generated or authored vessel with its own room graph,
  outer hull, interior spaces, and at least one **main airlock**.

- **Space Station**  
  A special multi-sided structure. For the initial implementation, a
  four-sided station has exactly four main airlocks, one on each compass
  side (North, East, South, West). The station may be generated as a
  larger central hub with internal rooms and systems.

- **Main Airlock**  
  A standardized two-tile-wide opening on a ship or station. It is the only
  valid connection point for stretchable umbilicals.

- **Umbilical**  
  A flexible, stretchable connector between two main airlocks. It is
  typically long enough to bridge the gap between docked entities. Gameplay
  actors can walk through it, and compatible systems can transfer across
  it.

- **Docking**  
  The process of connecting two ships/stations at their airlocks and
  merging their graphs into a single active world.

- **Composite Level**  
  The union of multiple ships and stations after docking. It is represented
  as one `RoomGraph`, one tile grid, and one set of system graphs.

---

## High-Level Composition Model

The composite level is built from a **docking plan**. A docking plan is a
list of ship/station instances and the airlocks that should be connected.

For the four-sided station:

```text
[Ship A] ---(U)--- 
                    \
[Station North]      [Station East]
       |                  |
[Station West]      [Station South]
       |                  |
[Ship C] ---(U)---  [Ship D] ---(U)---
```

More abstractly:

- The station is the **hub**.
- Four ships are the **spokes**.
- Each spoke connects to the station through exactly one umbilical.
- The station has one airlock per side.
- Ships have exactly one main airlock that faces the station.

The result is a single connected level with no overlapping geometry because
each side connection is aligned and spaced.

---

## Required Components and Data Structures

### DockingPlan

```rust
struct DockingPlan {
    instances: Vec<ShipOrStationInstance>,
    connections: Vec<DockingConnection>,
}
```

### ShipOrStationInstance

```rust
enum ShipOrStationInstance {
    Ship(ShipInstance),
    Station(StationInstance),
}

struct ShipInstance {
    template_id: String,
    world_position: Vec3,
    rotation: f32,
    main_airlock_local_pos: Vec3,   // relative to ship origin
}

struct StationInstance {
    template_id: String,
    world_position: Vec3,
    rotation: f32,
    airlocks: [Vec3; 4],            // North, East, South, West local positions
}
```

### DockingConnection

```rust
struct DockingConnection {
    from_instance: usize,   // index into instances
    from_airlock: AirlockId,
    to_instance: usize,
    to_airlock: AirlockId,
}
```

### AirlockId

A unique identifier within a given ship/station template.

### Generated Graph After Docking

After docking:

- All ship and station `RoomGraph`s are merged into a single `RoomGraph`.
- Rooms are relabeled with globally unique `RoomId`s.
- Internal edges are preserved.
- Docking connections become new `RoomEdge`s of type `EdgeKind::Umbilical`.
- World-space positions are stored as part of the merged graph, or the
  graph nodes carry a world-space transform offset that can be used
  during spawn.

---

## Stretchable Umbilical Model

Umbilicals are not rigid cubes. They are represented as a **flexible
corridor** between two airlocks.

### Geometric Representation

For implementation simplicity, the umbilical occupies a bounding rectangle
or swept path between the two airlock openings. The exact length is
computed from the world positions of the airlocks.

If the station is placed at the origin and ships are placed at cardinal
offsets:

```text
North ship airlock: (0, 0,  station_radius + ship_radius + gap)
Station north airlock: (0, 0, station_radius)
```

The umbilical spans the gap along the Z axis. Its length is the distance
between the airlock centers. If ships are not perfectly aligned, the
umbilical may include a straight or slightly angled path. For the initial
plan, all connections are axis-aligned, so an umbilical is a straight
two-tile-wide corridor.

### Tile Representation

In the merged `RoomGraph` and generated tile grid, the umbilical is
represented as:

- A `RoomEdge` of kind `EdgeKind::Umbilical`.
- A corridor area of passable `TileType::Floor` or a dedicated
  `TileType::UmbilicalFloor` (if needed later).
- Walls/dead zones around the umbilical if it is exposed to space. For
  now, the umbilical is an enclosed tube: its two side walls are generated
  as `TileType::Wall(WallDirection::*)` and the ceiling is closed.

The umbilical may also have a dedicated `UmbilicalComponent` for visual
stretching, but the gameplay layout is fixed at generation time.

### Stretchability

"Stretchable" means the umbilical length adapts to the distance between
the two airlocks. This is achieved during **docking graph generation**,
not during runtime. The system computes the required length and generates
the intermediate tiles or corridor accordingly.

If ships are placed too far apart, the umbilical may exceed a configured
maximum length (`max_umbilical_length`). In that case, the docking plan is
rejected or ships are pulled closer before docking.

---

## Four-Sided Space Station Layout

The station is generated as a central hub, often larger than a ship. It
contains:

- Four main airlocks, one per side.
- Internal rooms for engineering, life support, docking control, etc.
- A central `Bridge` or `Command` room that provides ship-wide system maps
  for the entire composite level.

### Dimensions and Placement

Let:

- `station_radius` = half the side length of the station (for a square
  station) plus an outer corridor margin.
- `ship_radius` = half the longer side of a ship (or a conservative
  bounding radius).

Ships are placed at cardinal offsets:

```text
North ship: (0, 0,  station_radius + ship_radius + gap)
South ship: (0, 0, -station_radius - ship_radius - gap)
East ship:  ( station_radius + ship_radius + gap, 0, 0)
West ship:  (-station_radius - ship_radius - gap, 0, 0)
```

`gap` is a fixed separation distance, typically one or two tiles. This
keeps hulls from overlapping and gives room for the umbilical.

### Airlock Alignment

- The station’s north airlock is on its north face, centered on X.
- The north ship’s airlock must face south, aligned with the station’s
  airlock.
- Similarly for east/west.

When docking, the combined bounding box of all instances is used as the
final level bounds. The entire composite level is centered at the world
origin for runtime player comfort.

---

## Collision Avoidance

The level generator must ensure that no ship/station geometry overlaps.
This is enforced by:

1. **Placement rules**  
   Each instance gets an extended bounding box that includes its outer
   hull plus a safety margin. The docking planner rejects any placement
   where these boxes intersect.

2. **Umbilical corridor**  
   The umbilical width is exactly two tiles. It must not cross any other
   ship’s hull or station structure. Since all connections are straight
   and axis-aligned, this is easy to validate.

3. **Runtime physics**  
   Colliders for ships, stations, and umbilicals are generated at spawn
   time. Since layouts are non-overlapping, physics broadphase will not
   report erroneous contacts. The generated tile grid is guaranteed to be
   non-intersecting.

Moreover, the station’s four airlocks are on different sides, so there is
no risk of two umbilicals crossing each other.

---

## Graph Merge and System Compatibility

When ships dock, their `RoomGraph`s are merged. For a four-sided station:

- The station graph becomes the central connected component.
- Each ship graph is connected to the station through one new
  `RoomEdge` of kind `EdgeKind::Umbilical`.
- The merged graph remains planar and connected.

System compatibility rules from the architecture still apply:

- An umbilical edge carries **only systems that both sides support**.
- For example, if a ship has `ELECTRICAL` and `ATMOSPHERICS`, and the
  station supports both, the umbilical edge will carry both. If one side
  lacks `FLUIDICS`, the edge does not carry fluidics.
- The `derive_system_graphs` function (or its future equivalent) must
  consider umbilical edges in addition to internal edges.

For the station, it is initially assumed to support at least `ELECTRICAL`
and `ATMOSPHERICS`, since the station is a large hub. It may also support
`FLUIDICS`, `THERMAL`, and `COMPUTER`, depending on generated rooms.

---

## Spawning and Runtime Details

The world builder for a composite level follows the same pattern as a
single ship, but with multiple input templates and a docking plan.

### Step-by-Step

1. **Generate all ship and station graphs**  
   For each ship/station template, run the existing planar scaffold + WFC
   generator to produce a local tile grid and `RoomGraph`. However, for
   the station, use a special station generator or template that includes
   four airlocks.

2. **Place instances in world space**  
   Apply the docking plan’s world-position and rotation. Record the
   world-space positions of each graph node.

3. **Compute umbilicals**  
   For each connection, compute the gap between the two airlocks.
   Generate a two-tile-wide passable corridor. Optionally add side walls
   to enclose the umbilical. Extend the local tile grid for each ship
   with these corridor tiles.

4. **Merge all tile grids into one global grid**  
   All local grids are placed into a single global grid based on world
   positions. If a cell belongs to multiple instances, that is a
   collision error and should be rejected.

5. **Merge all `RoomGraph`s**  
   Relabel room IDs, merge internal edges, and add `Umbilical` edges
   between the airlock rooms of the connected instances.

6. **Generate `CurrentRoomGraph` and `CurrentSystemGraphs`**  
   Store the merged graph and derived system graphs using the same
   resource types as single-ship generation.

7. **Spawn entities**  
   For each tile in the global grid, spawn floors, walls, doors, conduits,
   pipes, etc., as already done for single ships. Rooms are spawned from
   the merged graph. Docking ports are still `DockingPort` entities, but
   for already-docked ships they are replaced or marked as connected.

8. **Docking state initialisation**  
   `Scenario` or `DockingPlan` sets initial docking links. After the world
   is built, those docks are considered established. The game may later
   allow dynamic undocking and re-graphing.

---

## Editor and Authoring

In the in-game editor, the player should be able to:

- Place a new ship or station instance.
- Drag it near an existing airlock.
- When two compatible airlocks are within `max_umbilical_length`, offer
  a “docking snap” that aligns the airlocks and creates the umbilical.
- Move an already docked ship: the umbilical stretches or retracts until
  the new distance is within bounds.

To enable this, the editor must use the same placement validation and
docking graph logic as the headless generator.

For Eldiron-authored scenarios, a `DockingPlan` can be serialized as part
of `Scenario` or a separate `StationDefinition`. The initial
implementation will likely use a hardcoded four-sided station scenario
for testing.

---

## Validation and Map Contracts

The map assertion helpers must be extended with composite-level checks.

Examples:

- `passable_connected` across all ships and station.
- `room_graph_connected` after merging.
- No overlapping rooms from different instances.
- No orphan airlocks.
- Every main airlock is either connected or intentionally empty.
- Umbilical width is exactly two passable tiles.
- System graphs correctly prune unsupported systems across umbilical
  edges.
- The station has exactly four airlocks for the four-sided case.

A BDD-style contract for the four-sided case:

```gherkin
Given a four-sided station and four ships
When the docking plan is executed
Then the merged room graph is connected
And all four ships are reachable from the station through its airlocks
And no ship geometry overlaps the station or another ship
And each umbilical is a two-tile-wide passable corridor
```

---

## Implementation Steps

### Phase 1: Hardcoded Four-Sided Station Test Case

- Add a function `generate_four_sided_station_config()` that returns a
  `DockingPlan` with one station and four default ships.
- Generate the station template with four airlocks.
- Use simple rectangular ships with one airlock each.
- Place the ships at cardinal offsets.
- Generate the merged tile grid and room graph.
- Write map contract tests proving connectivity and no overlap.

### Phase 2: Generic Docking Planner

- Generalize the placement and umbilical generation logic.
- Accept arbitrary number of instances and connections.
- Add validation for overlapping bounding boxes and max umbilical length.

### Phase 3: World Spawning Integration

- Modify `spawn_ship_from_template` or create a new
  `spawn_composite_world_from_plan` that consumes a `DockingPlan`.
- Reuse per-tile entity spawning and graph resource setup.
- Ensure `CurrentRoomGraph` and `CurrentSystemGraphs` represent the merged
  graph.

### Phase 4: Editor and Scenario Authoring

- Extend scenario data to include docking plans.
- Add editor tools for placing stations and ships and snapping airlocks.
- Save/load composite scenarios.

### Phase 5: Dynamic Docking/Undocking

- Later, implement runtime undocking: when a player severs a dock, the
  world must be rebuilt partially. This is out of scope for initial
  composite-level generation.

---

## Non-Goals for This Plan

- No automatic pathfinding across arbitrary docking graphs during runtime;
  the merged graph is sufficient for current prototype.
- No procedural generation of the station interior beyond the standard
  room scaffold + WFC.
- No multi-level / vertical stacking of ships; all ships are on the same
  XZ plane.

---

## Summary

The four-sided space station is a composition of five separately generated
entities connected by stretchable umbilical corridors. The implementation
will reuse the existing room scaffold + WFC generation and graph merging
principles. The immediate next step is to add a hardcoded test scenario
that generates the station, four ships, and all four umbilicals, then
assert that the resulting map is connected, collision-free, and system
graphs are valid.

This plan establishes the foundation for more complex multi-ship levels
and dynamic docking in the future.
