# Ray, Light, and Sensor Mission Plan

## Purpose

This document is the focused implementation plan for raycasting, lighting,
sensor visibility, and the live fog of war in **Spacetrash v11**.

The architecture has shifted from dense per-frame raycasts to a shared
**cellular-automaton sensor grid** with `LightEmitter` / `LightReceiver`.
Raycasting remains as a low-level backend for specific queries, but the
primary visibility model is grid-based.

## Goals

- Provide a shared raycast backend for line-of-sight queries.
- Enforce a strict single ray-instrument limit per agent.
- Treat a visible-light camera as a `LightReceiver`, not as a special
  two-ray system.
- Support non-visible sensors such as LIDAR, thermal, and radar as single
  ray instruments that interact with the sensor grid.
- Build a live fog of war driven by cellular automaton propagation and
  decay.

## Current State

The prototype currently has no formal raycasting or visibility system. It
only has:

- a global directional light
- a single 3D camera
- AABB colliders for walls and player drones
- no `LightGrid`, no sensor registry, no visibility cache
- no sensor grid

This document defines the work from scratch.

## Required Components and Resources

### Components

- `GameLight`
- `GameCamera`
- `SensorEmitter`
- `VisibleLightSource`
- `Spectrum`
- `SensorOutput`
- `CircleCollider`
- `AabbCollider`
- `LineSegmentCollider`
- `LightEmitter` (new)
- `LightReceiver` (new)

### Resources

- `VisibilityCache`
- `LightGrid`
- `SensorRegistry`
- `RaycastBackend`
- `SensorGrid` (new, tile-based cellular automaton)
- `SensorGridBounds` (world-space mapping)

### Enums

- `LightKind`:
  - `Point`
  - `Spot`
  - `Directional`
- `Spectrum`:
  - `Visible`
  - `Infrared`
  - `Thermal`
  - `Radar`
- `SensorUpdateMode`:
  - `Continuous`
  - `Sweep`
  - `Ping`
- `FogState`:
  - `Unseen`
  - `CurrentVisible`
  - `RecentlyVisible`
  - `Blocked`

## Single Ray Instrument Constraint

Every agent is allowed at most one ray-based instrument at a time.

Examples:

- A visible-light 3D camera (a `LightReceiver`)
- A LIDAR
- A turret gun that uses a ray for aiming
- A thermal imager
- A radar emitter/receiver

This constraint is a core performance and design baseline. It prevents
expensive per-agent multi-ray operations and keeps sensor behavior simple.

A visible-light camera is **not** a special two-ray system. It is simply a
`LightReceiver` that samples visible light arriving at its position in the
shared sensor grid. The camera itself is the single ray instrument.

This does not prohibit an agent from having multiple non-ray upgrades such
as a light upgrade, a microphone, or a movement upgrade.

## Shared Raycast Backend

The prototype should implement a small raycast backend that supports:

- ray vs AABB
- ray vs circle
- ray vs line segment

The backend must be deterministic and cheap enough for occasional use in a
50-room / 20-actor scene. The primary visibility system will not call this
backend every frame; it will use the sensor grid.

### Ray vs AABB

Use slab method or closest-point method. For 2D gameplay in XZ plane, ignore
Y for logical queries.

### Ray vs Circle

Use standard ray-circle intersection.

### Ray vs Line Segment

Use a standard 2D segment intersection test. For future interior diagonal
tiles, slopes `0`, `∞`, `1`, and `-1` can be special-cased for speed.

## Light Emitters and Receivers

A clean ECS abstraction:

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

- `LightEmitter` seeds the shared `LightGrid`.
- A propagation system spreads light through transparent cells according to
  obstacle and attenuation rules.
- `LightReceiver` samples the `LightGrid` at its current cell.

## Sensor Grid

The sensor grid is a tile-based structure aligned with the room/tile graph.
Each cell stores:

- fog state
- light intensity
- thermal value
- radar ping timestamp
- last sensor update tick

Sensors update local regions and frequencies, not dense raycasts.

### Propagation

- Light: flood-fill through transparent cells.
- Thermal: diffuse through conductive cells.
- Radar: wavefront propagation with ping interval.
- LIDAR: structural wavefront propagation, no light requirement.

This shared grid allows many receivers to reuse one propagation pass,
amortizing cost.

## Focused View and View Upgrades

A focused view is supplied by a view upgrade.

### Visible-Light Camera

- A `LightReceiver` with `Spectrum::Visible`.
- Samples the local light grid.
- In darkness, the view is black/empty.
- The camera is the one allowed ray instrument.

### LIDAR

- A single ray instrument.
- Samples the structural wavefront of the sensor grid.
- Does not require visible light.
- Provides a top-down point cloud or line representation.

### Thermal Sensor

- A single ray instrument.
- Samples the thermal grid.
- Produces a heatmap.
- Blocked by insulated cells.

### Radar

- A single ray instrument.
- Samples the radar ping cell timestamps.
- Information is stale between pings.
- Good for detecting moving entities.

## Live Fog of War

The player’s ability to see is determined by the sensor grid.

- There is no persistent explored-map state.
- Each cell has a fog state that is updated by active sensors.
- Cells decay from `CurrentVisible` to `RecentlyVisible` to `Unseen`
  when no longer updated.
- The decay rate can be defined per sensor type.

## Rendering Implications

- The world camera visual feed is only one possible focused view.
- UI overlays such as schematic maps must not affect the world camera.
- Sensor overlays may be drawn using UI or custom shaders.
- The 3D world camera should remain hidden when the active view is not a
  visible-light camera view.

## Raycasting and Collider Geometry

Raycasting should reuse the same collider components as physics, primarily
for the raycast backend used by specific queries (e.g., tool checks,
AI line-of-sight confirmation). The sensor grid itself uses graph edges
rather than raw collider geometry.

This avoids duplicating collision geometry for visibility.

## Performance Budget

- Shared sensor grid amortizes visibility cost.
- Single ray instrument per agent caps per-actor raycast use.
- Sensor grid propagation is O(cells * neighborhoods) per tick, not
  O(actors * rays).
- FSA updates at low frequency.
- Spatial hash broadphase for physics.
- Deterministic headless mode for testing.

## Implementation Steps

### Step 1: Define ray-related components/enums

Add `LightEmitter`, `LightReceiver`, `SensorEmitter`, `Spectrum`,
`LightKind`, `SensorUpdateMode`, `FogState`, and `SensorOutput`.

### Step 2: Add `RaycastBackend` resource

Implement a basic 2D ray vs AABB/circle/segment query service.

### Step 3: Build `SensorGrid` resource

Create a tile-based structure matching the room/tile graph.

### Step 4: Implement light propagation

Add `LightGrid` propagation seeded by `LightEmitter` components.

### Step 5: Implement `LightReceiver` sampling

Add a system that updates `current_illumination` from the local light grid.

### Step 6: Add sensor grid propagation for other spectra

Implement thermal, radar, and LIDAR propagation as cellular automata.

### Step 7: Add fog-of-war decay

Update `FogState` per cell based on sensor activity and decay timers.

### Step 8: Integrate with focused view system

The focused view system should read `ActiveViewUpgrade` and select the
correct view representation, querying the sensor grid rather than
performing raw raycasts.

## BDD Scenarios

### Scenario: Visible camera requires light

- Given a drone with a visible-light camera ray instrument
- And no `LightEmitter` in visible range
- When the player focuses that drone
- Then the visible feed is dark or heavily obscured
- And the HUD indicates no visual light

### Scenario: LIDAR does not depend on light

- Given a drone with a LIDAR ray instrument
- And no visible light source in range
- When the player focuses that drone
- Then the LIDAR view still renders geometry from the structural wavefront
- But the view is top-down and rotating with the drone

### Scenario: Radar provides stale pings

- Given a drone with a radar ray instrument
- When the drone moves
- Then new radar pings occur on a fixed interval
- And between pings the displayed entity positions remain from the last
  sensor grid update

### Scenario: Live fog of war loses visibility when agent leaves

- Given a visible-light camera agent is present in Room A
- Then Room A cells are `CurrentVisible`
- When the camera moves to Room B
- Then Room A cells decay to `RecentlyVisible` or `Unseen`
- And Room B cells become visible if the camera has line of sight

### Scenario: Agent cannot have two ray instruments simultaneously

- Given an agent with a visible-light camera
- When an attempt is made to equip a LIDAR or turret gun ray instrument
- Then the previous ray instrument is unequipped or the new one is rejected
- And the agent still has exactly one active ray instrument

## Automated Testing of Ray Instrument Constraints

Automated arena playtesting must verify and exploit the single ray instrument
constraint.

### Validation in Test Generation

When generating a drone or enemy configuration, the assembler must ensure:

- exactly zero or one active ray instrument
- if a ray instrument is present, its component is correctly mapped
- all other sensory upgrades are non-ray or inactive
- no ray instrument is duplicated by conflicting upgrade modules

If the generated configuration violates the constraint, it is invalid and
should be rejected before simulation.

### Behavioral Descriptors

Record ray-specific behavioral descriptors:

- ray instrument type and usage frequency
- whether the agent ever switches or unequips a ray instrument
- time spent blind
- sensor detections per room
- line-of-sight failures due to darkness or occlusion
- reliance on non-ray sensors when ray instrument is absent

These descriptors help the search determine whether the single ray
constraint creates meaningful choice.

### Search Goal

A good ray-balance outcome is one where:

- `VisibleCamera` is strong when light is available, weak in darkness.
- `Lidar` is strong geometrically but lacks color/identity.
- `Thermal` detects powered/hot enemies but misses cold objects.
- `Radar` detects moving targets but not static loot or structural detail.
- no ray instrument is still viable with careful movement and local non-ray
  senses.

Search should reject configurations where any single ray instrument
dominates all others, or where no ray instrument is strictly inferior in a
way that creates unplayable blind conditions.

### BDD Scenario for Automated Ray Test

#### Scenario: Single ray constraint survives random assembly

- Given a random actor assembler with upgrade pool containing multiple
  ray instruments
- When it generates 10,000 actor configurations
- Then every generated actor has at most one active ray instrument
- And no configuration contains two conflicting ray components

#### Scenario: Blind agent can still navigate simple arena

- Given a drone with no ray instrument and only non-ray proximity/microphone
  upgrades
- And a small arena with few obstacles
- When the drone is controlled by a simple heuristic toward a goal
- Then it should reach the goal within a reasonable time without direct
  spatial sensor data

These checks ensure the ray architecture remains a real constraint rather
than an afterthought.
