# Upgrade System Design

This document describes the modular upgrade system that drives actor
capabilities in **Spacetrash v11**. It is the foundation for both the
player’s drone fleet and procedurally generated enemies.

The ultimate goal is that an actor is not a fixed class or species. An actor
is a specific collection of upgrades, each of which contributes concrete
components, behaviors, and capacities to the entity. Some upgrades are
permanently integrated into a chassis; others are swappable during play.
The same shared upgrade pool is used for player units, NPCs, monsters,
robots, and even future human crew members.

This symmetrical design has two main benefits:

1. It keeps player and enemy mechanics on the same code path.
2. It makes procedural enemy generation an exercise in assembling weighted
   upgrade sets rather than hand-authoring dozens of separate monster types.

The original **Duskers** relied on a small number of fixed species and fixed
drone tools. The original enemy species had only limited feature sets and
very little interaction. Spacetrash instead treats every actor, whether
obedient fleet drone or hostile derelict monster, as an instance constructed
from the same upgrade vocabulary.

---

## 1. Core Model

An actor is defined by:

- a **chassis**
- a set of **upgrade modules**
- runtime state generated from those upgrades

### Chassis

The chassis is the base physical body of the actor. It provides:

- base collider shape and size
- base movement profile
- base health/energy/armor
- available upgrade slots
- any fixed built-in components

Chassis are shared across factions and can be reused to create different
actors.

Examples:

- `LightDroneChassis`
- `StandardDroneChassis`
- `HeavyDroneChassis`
- `HumanCrewChassis`
- `MonsterSmallChassis`
- `MonsterLargeChassis`

### Upgrade Module

An upgrade module is a named, data-driven package that adds one or more
components, attributes, or behaviors to the actor.

Upgrades may be:

- **fixed / permanent** — built into the chassis at spawn time
- **swappable** — can be installed or removed during play
- **activated / passive** — some upgrades need player or AI activation
- **ray instrument** — at most one active ray-based upgrade per actor

All upgrade definitions are authored in Eldiron and converted into Bevy
components when an actor is spawned.

---

## 2. Upgrade Categories

Upgrades are organized into broad categories. An actor may have multiple
upgrades from different categories, but some categories impose strict
constraints, especially ray instruments.

### 2.1 Movement Upgrades

Movement upgrades define how an actor moves.

Examples:

- `WheelDriveUpgrade`
- `LegsUpgrade`
- `HoverUpgrade`
- `TracksUpgrade`
- `ThrusterUpgrade`

Movement upgrades primarily map to:

- `PhysicsBody`
- `ActorSize`
- `CircleCollider`
- movement tuning values

Movement upgrades create different arcade feels:

- wheel drive = high friction, lower turn rate
- legs = balanced turning and acceleration
- hover = low friction, pronounced drift
- tracks = good linear power, poor turning
- thrusters = bursty movement with cooldown

### 2.2 Sensory Upgrades

Sensory upgrades define how an actor perceives the world.

Examples:

- `VisibleCameraUpgrade`
- `LidarUpgrade`
- `RadarUpgrade`
- `ThermalUpgrade`
- `MicrophoneUpgrade`
- `MotionDetectorUpgrade`

Sensory upgrades map to:

- `Spectrum`
- `SensorEmitter`
- `SensorUpdateMode`
- `GameCamera`
- `VisibleLightSource` or other light coupling components

Only one sensory upgrade may be an active ray instrument at a time. However,
multiple non-ray senses such as microphones or passive motion detectors may
be installed simultaneously.

### 2.3 Effector / Weapon Upgrades

Effector upgrades allow the actor to act on the world.

Examples:

- `TurretGunUpgrade`
- `MeleeArmUpgrade`
- `GrabberUpgrade`
- `RepairToolUpgrade`
- `LightUpgrade`
- `ExtinguisherUpgrade`
- `HackingModuleUpgrade`

Weapon effectors usually require a raycast for targeting if they are
line-of-sight weapons. A turret gun that uses ray-based aiming counts as a
ray instrument. Melee arms and grabbers do not have the same global ray
budget because they operate over short, local ranges.

### 2.4 Defensive Upgrades

Defensive upgrades affect survivability.

Examples:

- `ArmorPlateUpgrade`
- `EnergyShieldUpgrade`
- `FireproofPlatingUpgrade`
- `RadiationShieldingUpgrade`

These map to health, armor, shields, resistances, and sometimes visual
changes.

### 2.5 Utility Upgrades

Utility upgrades provide environmental or inventory-related abilities.

Examples:

- `BatteryUpgrade`
- `CargoBayUpgrade`
- `ScannerUpgrade`
- `PowerCellUpgrade`
- `TractorBeamUpgrade`

Some utility upgrades may consume more energy or increase weight, creating
build trade-offs.

### 2.6 Behavior Upgrades

Behavior upgrades are the most important category for procedural AI.

Instead of a monolithic `EnemyType`, an actor has one or more behavior
upgrades that influence how systems decide what to do.

Examples:

- `PatrolBehavior`
- `GuardBehavior`
- `AggressiveBehavior`
- `FleeBehavior`
- `ScavengeBehavior`
- `FollowLeaderBehavior`
- `HuntBehavior`
- `ProtectBehavior`

Behavior upgrades map to small components, not hardcoded AI classes. Multiple
behavior components may coexist, and a behavior arbitration system chooses
the active behavior based on priorities and current situation.

Behavior upgrades are covered in more detail later in this document.

### 2.7 Status Modifier Upgrades

Status modifiers provide numerical bonuses or penalties.

Examples:

- `HealthBoost`
- `EnergyDrain`
- `MovementSlow`
- `DamageAmp`
- `NoiseReduction`

They are often simple attribute changes rather than full components.

---

## 3. Ray Instrument Constraint

Every actor is allowed **one active ray-based instrument** at a time.

This applies to upgrades that drive frequent raycast operations:

- visible-light camera
- LIDAR
- turret gun using ray-based aiming
- thermal imager
- radar emitter/receiver

A visible-light camera is not a special two-ray system. It is simply a
light-ray-detecting 3D camera. It receives visible light along rays and is
useless in total darkness unless the actor has its own light source or
another actor provides light.

This constraint does not prohibit non-ray upgrades such as:

- lights
- microphones
- motion detectors
- movement upgrades
- armor
- utility modules

If an actor attempts to equip or activate a second ray instrument, the
previous ray instrument must be unequipped first, or the new upgrade is
rejected.

---

## 4. Actor Definition

In ECS terms, an actor is an entity with components derived from its
upgrades.

At spawn time, Eldiron data or procedural generation decides:

- which chassis to use
- which fixed upgrades are integrated into the chassis
- which swappable upgrades begin installed
- which upgrade slots remain empty

The resulting entity is assembled from:

- base chassis components
- one or more movement components
- at most one active sensory ray instrument
- zero or more non-ray sensors
- zero or more behavior components
- zero or more effectors
- zero or more defensive/utility upgrades

This assembly is symmetrical between player and enemy units.

---

## 5. Player Fleet as Upgrade Sets

The player’s fleet is defined using the same upgrade system.

The Blackbox is not simply a special character. It is a chassis that carries
command-and-control upgrades and may be transported by other drones. The
Blackbox does not have its own camera perspective unless it is carried by
another drone or equipped with an active camera ray instrument.

Each player-controlled drone has:

- a `Drone { index }` component
- a chassis
- a set of installed upgrades
- one active ray instrument, if any
- one or more behavior upgrades if it is to act autonomously
- no inherent AI unless a behavior upgrade grants one

Player drones may be:

- **obedient fleet bots** — direct player input controls movement
- **semi-autonomous drones** — behavior upgrades such as `FollowLeader`,
  `GuardLocation`, or `Scavenge` allow the drone to perform tasks without
  frame-by-frame player input

The fleet configuration view opened by key `0` exists to show:

- chassis type
- installed upgrades
- available upgrade slots
- power/energy status
- inventory capacity
- active ray instrument
- any installed behavior upgrades

It does not show live sensor data. It is a management screen, not a viewing
feed.

---

## 6. Procedural Enemy Generation

Enemies are procedural actors assembled from the shared upgrade pool.

An enemy generator selects:

- a chassis
- one or more movement upgrades
- sensory upgrades, with at most one ray instrument
- one or more behavior upgrades
- weapon/effector upgrades
- defensive upgrades
- optional status modifiers

The goal is not to create “species” with hardcoded personalities. It is to
create robots or monsters with coherent upgrade combinations. Coherence is
controlled by tags, weights, and prerequisites.

For example, a heavily armored enemy should be more likely to have a
slow movement chassis and a close-range melee weapon. A fast scouting enemy
should be more likely to have a visible-light camera and low armor.

---

## 7. Behavior Upgrades in Detail

Behavior upgrades are the core of emergent AI.

Each behavior upgrade maps to a component that can be read by AI systems.

### Suggested Behavior Components

#### `PatrolBehavior`

- `waypoints: Vec<Vec2>`
- `loop: bool`

The actor moves from waypoint to waypoint.

#### `GuardBehavior`

- `guard_position: Vec2`
- `radius: f32`

The actor stays near a position and reacts to perceived intruders.

#### `AggressiveBehavior`

- `engage_range: f32`
- `weapon_range: f32`

The actor approaches and attacks targets detected by its sensory upgrades.

#### `FleeBehavior`

- `health_threshold: f32`
- `preferred_distance: f32`

The actor moves away from threats when health drops below the threshold.

#### `ScavengeBehavior`

- `target_priorities: Vec<ResourceType>`

The actor searches rooms for loot and attempts to collect or interact with
resources.

#### `FollowLeaderBehavior`

- `leader_entity: Entity`
- `follow_distance: f32`

The actor follows a designated leader, usually the focused fleet drone or
Blackbox.

#### `HuntBehavior`

- `target_tags: Vec<String>`
- `memory_duration: f32`

The actor remembers and hunts a specific target even if line of sight is
temporarily lost.

#### `ProtectBehavior`

- `protected_entity: Entity`

The actor places itself between the protected entity and perceived threats.

### Behavior Arbitration

An actor may have more than one behavior upgrade installed.

A `BehaviorArbitration` system chooses which behavior component should be
active at any moment. Priority is data-driven and situational.

A sensible priority order is:

1. `FleeBehavior` when health is below threshold and threat is present
2. `AggressiveBehavior` or `HuntBehavior` when a visible valid target exists
3. `ProtectBehavior` when the protected entity is threatened
4. `ScavengeBehavior` when loot is detected and no immediate threat exists
5. `GuardBehavior` when the actor is within its guard radius
6. `FollowLeaderBehavior` when no higher priority applies
7. `PatrolBehavior` as a fallback
8. passive/no behavior

Future behavior systems may evaluate all installed behavior components and
choose the one that best matches the actor’s current sensory state.

---

## 8. Sensory and Ray Instrument Balance

Because every actor is limited to one ray instrument, sensory upgrades are a
meaningful strategic choice.

A visible-light camera provides the most detailed view but requires light.
A LIDAR provides geometry even in darkness but does not show color or
identify details.
A radar provides periodic moving-target pings but has stale information.
A thermal imager detects heat but lacks precise structural data.

Player drones can carry only one of these in the active ray slot. They may
also carry non-ray sensors such as microphones or simple proximity detectors.

Enemies should be generated with sensory packages that match their behavior.
For example:

- a hunter enemy should have a ray instrument that lets it detect targets
- a guard enemy may have only a short-range proximity sensor
- a scavenger may have a microphone or no ray instrument at all
- a blind enemy may have no ray instrument but still be dangerous in melee

---

## 9. Holistic Balancing of Procedural Actors

Balancing procedural enemies requires looking at all dimensions together:

- movement speed and maneuverability
- sensor range and reliability
- weapon range and damage
- armor and health
- behavior aggression
- ray instrument cost and limitations
- power consumption

Rather than balancing a fixed enemy type, balance tags and generation rules.

### Suggested Balance Tags

- `fast`
- `tanky`
- `stealthy`
- `blind`
- `ranged`
- `melee`
- `sensor_heavy`
- `sensor_light`
- `autonomous`
- `pack_hunter`
- `protector`
- `scavenger`

A generation profile selects tags and weights across categories.

For example, a `pack_hunter` profile might generate:

- chassis: fast light drone
- movement: legs or hover
- sensor: visible-light camera or radar
- behavior: `HuntBehavior`, `FollowLeaderBehavior`
- weapon: low-damage rapid turret
- armor: light
- special rule: one weapon, no heavy armor

A `guardian` profile might generate:

- chassis: heavy drone
- movement: tracks or wheel drive
- sensor: short-range thermal imager
- behavior: `GuardBehavior`
- weapon: strong melee arm
- armor: heavy
- special rule: no visible camera, slow movement

These profiles are authored as weighted generation tables, not as hardcoded
species.

---

## 10. Data-Driven Upgrade Definitions

Upgrades are not hardcoded in Rust. They are data.

Eldiron authors:

- upgrade module IDs
- display names
- slot categories
- component mappings
- attribute changes
- ray instrument flag
- energy cost
- weight
- compatibility tags
- behavior priorities
- generation weights

Bevy loads these definitions into a central upgrade registry at startup.
Actor spawning uses this registry to assemble the correct components.

This keeps the upgrade system moddable and extensible without changing core
game code.

---

## 11. Component Mapping

Each upgrade definition should include a machine-readable mapping to one or
more ECS components.

For example:

```text
Upgrade: VisibleCameraUpgrade
Component Mapping:
  -> GameCamera
  -> Spectrum::Visible
  -> SensorShape::Cone
  -> SensorUpdateMode::Continuous
  -> ray_instrument = true
```

```text
Upgrade: WheelDriveUpgrade
Component Mapping:
  -> MovementProfile
  -> PhysicsBody
  -> CircleCollider
  -> rotation profile(s)
```

```text
Upgrade: PatrolBehavior
Component Mapping:
  -> PatrolBehavior
  -> AiBehavior::Patrol
```

This mapping allows the same authored upgrade to be understood by both the
player fleet UI and procedural enemy generator.

---

## 12. Player and Enemy Symmetry

Player robots and enemy monsters use the same chassis and upgrade systems.

That means:

- a player can learn how an enemy behaves by observing the combination of
  visible upgrades
- player drones can equip many of the same upgrades found on enemies
- a captured enemy chassis may be repurposed later
- behavior upgrades may be installed on player drones to grant autonomous
  behavior
- an enemy can theoretically be generated from the same chassis as a player
  drone

This symmetry is a core design value. It enables emergent tactical depth
without multiplying special-case code.

---

## 13. Immediate Implementation Focus

The current prototype has only stubs for control and first-person camera
components. The upgrade system should be introduced incrementally.

Immediate next steps:

1. Define an `UpgradeDefinition` resource and registry.
2. Define a `Chassis` component that describes base actor properties.
3. Add an `UpgradeSlot` concept for swappable upgrades.
4. Convert the current player Blackbox and test drones into entities
   assembled from a minimal set of upgrade definitions.
5. Add a simple `BehaviorUpgrade` marker component and a `BehaviorArbitration`
   stub.
6. Extend the config view to display installed upgrades.
7. Build a minimal procedural enemy generator that selects from weighted
   profiles and spawns an enemy entity using the same assembler.

Full ray-instrument enforcement, detailed balance tags, and Eldiron-authored
upgrade data can follow once the ECS foundations are stable.

---

## 14. Future Work

Once the core upgrade system exists, the project can support:

- capture and reuse of enemy chassis
- hot-swapping of player drone upgrades during play
- research-gated upgrade modules
- behavior-upgrade combinations that produce emergent tactics
- AI leaders and followers that coordinate as a pack
- procedural enemy factions with coherent upgrade profiles
- visual customization based on chassis and upgrade tags
- modded upgrade packs loaded through Eldiron data

The underlying principle remains the same:

> An actor is the sum of its upgrades. Chassis provides a physical
> foundation, but every meaningful capacity, sensory mode, and behavior is
> an upgrade.
