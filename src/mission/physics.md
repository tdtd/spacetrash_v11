# Physics Mission Plan

## Purpose

This document is the focused implementation plan for the prototype arcade
physics layer in **Spacetrash v11**. It expands the broader mission plan and
architecture into concrete next steps, component/system boundaries, and
behavioral scenarios.

## Goals

- Provide lightweight continuous movement on the XZ plane.
- Support tank-style turning with WASD movement and QEZC rotation.
- Keep the prototype deterministic and predictable.
- Support up to 20 active actors and 50 rooms.
- Prepare for circle colliders, actor sizes, and future interior diagonal
  colliders.

## Current State

The prototype currently has:

- `Velocity(pub Vec3)`
- `AngularVelocity { yaw: f32 }`
- `Rotation { yaw: f32 }`
- `AabbCollider { half_extents: Vec3 }`
- `player_input_system` that reads WASD/QEZC and writes linear/angular velocity
- `movement_system` that applies damping, integrates rotation, and resolves
  AABB overlap against static walls

This is deliberately small and serves as the baseline for the full arcade
physics system.

## Required Components

### Already Present

- `Velocity`
- `AngularVelocity`
- `Rotation`
- `AabbCollider`

### To Add

- `PhysicsBody`:

  ```text
  mass: f32
  linear_damping: f32
  angular_damping: f32
  friction: f32
  restitution: f32
  max_linear_speed: f32
  max_angular_speed: f32
  ```

- `CircleCollider`:

  ```text
  radius: f32
  ```

- `LineSegmentCollider`:

  ```text
  start: Vec2
  end: Vec2
  ```

- `ActorSize`:

  ```text
  Small
  Standard
  Large
  ```

- Optional `CollisionEvent`:

  ```text
  entity_a: Entity
  entity_b: Entity
  normal: Vec3
  penetration: f32
  ```

## Actor Size Rules

- `Small`: no collision, can enter air vents.
- `Standard`: radius = `1 / 6` tile, diameter = `1 / 3` tile.
- `Large`: radius = `1 / 3` tile, diameter = `2 / 3` tile.

A one-tile door is exactly one world unit wide. Therefore:

- Two `Standard` actors may pass each other in a one-tile corridor or door.
- A `Large` actor cannot pass another `Standard` or `Large` actor in a
  one-tile corridor or door.

The collider radius is set at spawn time based on `ActorSize`.

## Input and Control

The current input scheme is fixed:

- `W`: forward
- `S`: backward
- `A`: strafe left
- `D`: strafe right
- `Q` or `Z`: rotate left
- `E` or `C`: rotate right

Movement is relative to the actor's current yaw. The goal is not pure
grid movement, but continuous tank-style movement with drift.

The input system must:

1. Build a local movement vector:

   ```text
   local = (strafe, 0, -forward)
   ```

2. Normalize if length squared is greater than zero.
3. Rotate local vector into world space using the actor's current yaw.
4. Apply linear acceleration using `PhysicsConfig.acceleration`.
5. Clamp world velocity to `PhysicsConfig.max_speed`.
6. Build a rotation input value:
   - left = positive yaw
   - right = negative yaw
7. Apply angular acceleration using `PhysicsConfig.angular_acceleration`.
8. Clamp angular velocity to `PhysicsConfig.angular_max_speed`.

## Integration Order

The movement system should use this per-frame order:

1. Read `Time.delta_seconds()`.
2. Apply linear damping.
3. Apply angular damping.
4. Clamp angular velocity.
5. Integrate yaw from angular velocity.
6. Update transform rotation from yaw.
7. Compute tentative next position from linear velocity.
8. Resolve collisions.
9. Write final transform translation.

## Collision Model

### Broadphase

Currently no broadphase exists. The immediate next step is a simple spatial
hash grid.

Cells should be at least twice the maximum actor radius. A reasonable cell
size for the prototype is `2.0` world units.

Static colliders are inserted into the broadphase at spawn time. Dynamic
actors are inserted/updated each frame after integration.

### Narrowphase

For the prototype:

- `Circle vs AABB`
- `Circle vs Circle`
- `Circle vs LineSegment` once interior diagonal colliders are introduced

`Circle vs AABB` can use the closest-point-on-box method:

```text
closest_x = clamp(circle_x, box_min_x, box_max_x)
closest_z = clamp(circle_z, box_min_z, box_max_z)
delta = circle_center - closest_point
penetration = circle_radius - delta.length()
```

`Circle vs LineSegment` uses the closest point on the segment and the same
circle penetration check.

`Circle vs Circle` checks the sum of radii against center distance.

### Resolution

For a dynamic circle hitting a static AABB or segment:

1. Compute penetration vector.
2. Push the dynamic circle out along the minimum translation axis.
3. Remove the velocity component along the contact normal to prevent jitter.
4. Do not apply bounce unless `restitution` is explicitly needed.

For dynamic vs dynamic:

1. Compute overlap.
2. Apply positional correction to both entities.
3. Optionally adjust velocities along the normal.

### Collision Events

Collision events should be emitted only when a new contact is detected with
relative velocity above a small threshold. This prevents event spam while
sleeping against a wall.

## Fake Physics Extensions

- **Drift / momentum**: partial rotation of existing velocity when turning.
- **Knockback**: direct velocity change on damage events.
- **Push / pull**: velocity manipulation, not forces.
- **Hazard triggers**: trigger colliders that fire events but do not resolve.

For the initial mission plan, only drift and knockback are explicitly
explored.

## PhysicsConfig Tuning

Current values:

```text
fixed_timestep: 1.0 / 60.0
max_speed: 4.0
acceleration: 20.0
damping: 10.0
angular_acceleration: 12.0
angular_max_speed: 8.0
angular_damping: 6.0
```

These are intentionally arcade-style. They should be tuned through playtests,
not assumed final.

## Performance Budget

- Fixed timestep: 60 Hz.
- Dynamic bodies: <= 20.
- Static rooms: <= 50.
- Broadphase: spatial hash with 2.0-unit cells.
- No multithreaded physics needed yet.
- Target physics cost should remain under 1 ms per frame.

## Implementation Steps

### Step 1: Add missing physics components

Add `PhysicsBody`, `CircleCollider`, `LineSegmentCollider`, `ActorSize`,
and a placeholder `CollisionEvent`.

### Step 2: Replace player AABB collider with CircleCollider

Use `CircleCollider { radius: 0.1667 }` for standard actors.

### Step 3: Add `PhysicsBody` to player and test drones

Use default values from `PhysicsConfig` for prototype.

### Step 4: Build a separate `physics_systems.rs`

Move player input integration and movement integration out of `systems/mod.rs`
into a dedicated module. Keep `player_input_system` for input mapping.

### Step 5: Implement broadphase spatial hash

Add a `PhysicsBroadphase` resource that stores dynamic entity references by
grid cell.

### Step 6: Implement narrowphase circle-vs-AABB

Replace the current AABB overlap resolution with circle-vs-AABB resolution.

### Step 7: Implement circle-vs-circle

Allow two controllable drones to collide with each other.

### Step 8: Add collision events

Emit events for initial contacts with sufficient relative velocity.

### Step 9: Add actor size filtering

Skip collision for `Small` actors. Derive collider radius from `ActorSize`.

### Step 10: Add line-segment colliders for future interior diagonal walls

Keep the collision backend ready, but no world content uses it yet.

## BDD Scenarios

### Scenario: Player moves forward with W

- Given the focused actor is a controllable standard drone
- When the player holds `W`
- Then the actor accelerates in its current facing direction
- And the actor does not strafe sideways
- And the actor does not exceed its maximum linear speed

### Scenario: Player rotates left with Q

- Given the focused actor is controllable
- When the player holds `Q`
- Then the actor's yaw increases positively
- And the actor eventually turns left
- And the actor does not move forward unless `W` is also pressed

### Scenario: Drone collides with a static wall

- Given a standard actor with a circle collider
- And a static AABB wall
- When the actor moves into the wall
- Then the actor is pushed out along the minimum penetration axis
- And the actor's velocity component along the contact normal is removed
- And no continuous bouncing occurs

### Scenario: Two standard drones pass in a one-tile door

- Given two standard drones with radius `1/6` tile
- And a one-tile-wide opening
- When both drones attempt to pass through the opening from opposite directions
- Then they should be able to pass using slight positional adjustments
- And neither drone becomes permanently stuck

### Scenario: Large drone cannot pass a standard drone in a one-tile door

- Given one large drone with radius `1/3` tile
- And one standard drone with radius `1/6` tile
- And a one-tile-wide opening
- When both attempt to pass through the opening
- Then the combined width exceeds the one-tile opening
- And the physics system should prevent them from passing through the same section simultaneously

## Deterministic Headless Physics for Automated Testing

Automated playtesting requires the physics layer to be strictly
deterministic and headless-runnable.

### Required Properties

- Fixed timestep with no variable frame time.
- Seeded random number generation; no wall-clock time.
- Deterministic world loading from a configuration hash.
- Same input sequence → same trajectory and collision events.
- No reliance on rendering, audio, or UI.
- No per-frame floating-point nondeterminism due to unordered iteration.

### Validation Checks Before Search

For every sampled configuration, run a physics sanity check:

- no collider penetration after resolution
- no NaN positions/velocities
- no actor moving faster than `max_speed`
- no actor rotating faster than `angular_max_speed`
- no continuous collision jitter
- no softlock caused by all colliders blocking

If any check fails, exclude the configuration from search scoring.

### BDD Scenario for Determinism

#### Scenario: Same seed gives identical physics trajectory

- Given two headless simulation runs with the same seed and configuration
- When both run for 600 fixed ticks
- Then all actor transforms and velocities match within floating-point epsilon
- And collision event logs match exactly

#### Scenario: Physics exploit is rejected

- Given a configuration where a circle passes through a wall due to a bug
- Then the physics sanity checker detects penetration
- And that configuration is flagged invalid

These checks prevent the search from amplifying physics bugs into false
“good behavior.”

## Immediate Focus

The immediate prototype focus is:

- Circle collider for players/drones.
- Circle-vs-AABB resolution.
- Actor size stub.
- Spatial hash broadphase scaffold.
- Keep movement deterministic and responsive.

No joints, torque simulation, or complex material interactions are planned.
