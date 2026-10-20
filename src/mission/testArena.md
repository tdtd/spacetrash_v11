# Test Arena Roadmap

## Goal

Build a deterministic headless arena runner that:

- samples upgrade-driven actor builds
- generates arenas with ship scaffolds and interiors
- runs fixed-tick episodes without rendering
- logs structured events
- validates simulation sanity
- extracts behavior descriptors
- searches for diverse, non-degenerate actor/level configurations using MAP-Elites

This is the longest-lead technical ambition in the project. Almost every
other system is a dependency of this one.

The keystone capability is determinism:

> Same arena config + same seed + same fixed tick count
> = identical final snapshot and identical structured event log.

All later search work depends on that guarantee.

---

## Phase 1 — Deterministic Headless Foundation

### Objective

Run a Bevy headless episode and prove that two identical configurations
produce identical results.

### Work

- Add `ArenaConfig`
  - seed
  - tick count
  - scenario/ship config
  - actor build specs
  - goal/metrics config

- Create a reusable headless arena runner.
- Replace unordered iteration where it affects output.
- Sort entities/events deterministically.
- Route all randomness through explicit seeded RNGs.
- Add `run_arena(config) -> ArenaRunResult`.

### Tests

- Same seed + config over 600 fixed ticks:
  - identical transforms
  - identical velocities
  - identical sensor states
  - identical event log

- Different seeds may produce different logs, but no nondeterminism.

### Acceptance

- A `cargo test` target can execute a headless episode and assert exact state.

---

## Phase 2 — Deterministic World Reset and Spawning

### Objective

Spawn a generated ship + actors predictably and reset it without leaks.

### Work

- Create an arena world builder that:
  1. generates `RoomGraph` and tile map
  2. spawns `GameplayEntity` ship geometry
  3. spawns controllable and NPC actors
  4. inserts resources, event log, physics state

- Implement `reset_arena(world)`.
- Ensure no leftover entities/resources after repeated reset.

### Tests

- `spawn -> record -> reset -> spawn` repeated 10 times:
  - first and fifth spawn produce identical state
  - no entity count drift
  - no stale `GameplayEntity` remains

### Acceptance

- World generation/reset is clean and deterministic.

---

## Phase 3 — Upgrade Registry and Actor Assembler

### Objective

Build actors from data-driven upgrade modules, not hardcoded archetypes.

### Work

- Define:
  - `ChassisDefinition`
  - `UpgradeDefinition`
  - `ActorBuildSpec`

- Add `UpgradeRegistry`.
- Add `ActorBuilder`.

- ActorBuilder:
  1. starts from chassis base components
  2. applies upgrade component mappings
  3. sets collider size
  4. applies camera/ray instrument
  5. applies behavior components

- Enforce **exactly one active ray instrument per actor**.

### Tests

- Builder produces:
  - drone with visible camera
  - NPC with thermal sensor
  - melee robot with no ray instrument

- Build with two ray instruments is rejected.
- Player and NPC from same upgrade spec receive equivalent components.
- Build output is deterministic from seed.

### Acceptance

- Actor generation is data-driven and validated before simulation.

---

## Phase 4 — Behavior FSA and Arbitration

### Objective

Give generated actors simple behavior components and deterministic arbitration.

### Work

- Add:
  - `AiBehaviorState`
  - `PatrolBehavior`
  - `GuardBehavior`
  - `AggressiveBehavior`
  - `FleeBehavior`
  - `ScavengeBehavior`
  - `FollowLeaderBehavior`

- Implement `BehaviorArbitration`.
- Implement small behavior systems that write desired velocity/action.
- Keep all transitions deterministic and fixed-timestep.

### Tests

- Priority order is deterministic:
  - Flee when low health and threat present
  - Aggressive when enemy visible
  - Scavenge when loot detected
  - Follow when leader present
  - Guard when in guard radius
  - Patrol as fallback

- Same sensor/perception inputs -> same behavior state every tick.
- Behavior systems produce expected movement direction/action.

### Acceptance

- Behavior is modular and deterministic.

---

## Phase 5 — Structured Event Log

### Objective

Capture enough data to compute behavior descriptors and validate simulations.

### Work

- Add `ArenaEventLog`.
- Define events:
  - `ActorSpawned`
  - `MovementTick`
  - `CollisionResolved`
  - `SensorDetection`
  - `DoorStateChanged`
  - `FsaTransition`
  - `SystemFailure`
  - `CascadeEvent`
  - `SalvageAction`
  - `RayInstrumentChanged`
  - `SoftlockSuspected`

- Each event includes tick, involved entities, and relevant values.
- Log every important simulation change.
- Avoid floating-point noise.

### Tests

- Known scenario produces known event sequence.
- Event log is serializable / comparable.
- Events sorted deterministically within a tick.
- Missing/duplicate events detected in simple cases.

### Acceptance

- Event log is sufficient to reconstruct behavior descriptors without
  inspecting live ECS state.

---

## Phase 6 — Arena Runner and Episode Lifecycle

### Objective

Run an episode headlessly from config to result.

### Work

- Define:

  ```rust
  struct ArenaEpisodeResult {
      config_hash: String,
      final_snapshot: ArenaSnapshot,
      event_log: ArenaEventLog,
      validity: Vec<ValidityError>,
      descriptors: BehaviorDescriptors,
  }
  ```

- Runner steps:
  1. load `ArenaConfig`
  2. build/reset world
  3. run N fixed ticks
  4. capture snapshot
  5. extract event log
  6. run validation checks
  7. compute behavior descriptors
  8. return result

- No rendering/UI/audio.

### Tests

- Same config hash produces identically structured result.
- Episode can run for multiple sampled actor builds.
- Invalid world setup is caught before simulation if possible.

### Acceptance

- A single function can run thousands of episodes without interactive input.

---

## Phase 7 — Simulation Sanity and Softlock Validation

### Objective

Separate broken simulations from interesting gameplay.

### Checks

- no NaN or infinite positions/velocities
- no collider penetration after resolution
- no actor speed > configured max
- no actor angular speed > configured max
- no continuous collision jitter
- no permanent softlock
- single ray instrument constraint preserved
- required rooms/goals reachable
- no unreachable FSA states if configured

### Tests

- Inject:
  - NaN position after a tick
  - overlapping colliders after resolution
  - softlocking actor behavior

- Validation flags all three as invalid.

### Acceptance

- Search will not amplify physics bugs or degenerate builds.

---

## Phase 8 — Behavior Descriptor Extraction

### Objective

Compute compact, meaningful descriptors from the event log.

### Descriptors

- active behavior state frequency histogram
- ray instrument type usage
- time spent blind
- sensor usage per type
- number of distinct rooms entered
- movement style:
  - direct
  - reactive
  - roaming
  - stationary
  - oscillating
- combat avoidance vs engagement
- resource extraction rate
- cascade event count
- system failure count
- softlock count
- failed action count
- FSA transition frequency

### Work

- Implement:

  ```rust
  fn extract_descriptors(
      event_log: &ArenaEventLog,
      final_snapshot: &ArenaSnapshot,
      goal: &ArenaGoal,
  ) -> BehaviorDescriptors
  ```

- Normalize descriptors to episode length.
- Ensure deterministic order and numeric stability.

### Tests

- Hand-crafted event log yields expected descriptors.
- Same event log -> same descriptor vector.
- Extraction does not depend on entity iteration order.

### Acceptance

- Descriptors become the basis for MAP-Elites and quality measurement.

---

## Phase 9 — Random Sampled Search Baseline

### Objective

Run many random actor/level configs and measure descriptor diversity.

### Work

- Create sampler for:
  - random upgrade combinations
  - random chassis
  - random initial system states
  - random arena layouts

- Run N random episodes.
- Filter invalid episodes.
- Build descriptor archive.
- Inspect for dominance, degenerate clusters, no diversity.

### Tests

- 100 random episodes produce multiple distinct descriptor clusters.
- Invalid episodes do not enter archive.
- Results reproducible from sampler seed.

### Acceptance

- A baseline search exists without MAP-Elites overhead.

---

## Phase 10 — MAP-Elites Search

### Objective

Evolve diverse, high-quality actor/level configurations.

### Work

- Choose 2–4 behavior descriptor dimensions as MAP-Elites grid:

  Example:
  - movement style
  - time blind fraction
  - system interaction frequency
  - combat engagement vs avoidance

- Implement:
  - `elite_grid: HashMap<DescriptorBin, Candidate>`
  - mutation/recombination of `ActorBuildSpec` and `ArenaConfig`

- Quality reward:
  - purposeful non-degenerate movement
  - meaningful response to threats/resources
  - recoverable cascades
  - multiple viable tactics
  - not just win rate

- Add experiment runner that outputs best archive.

### Tests

- Fixed-seed MAP-Elites run:
  - multiple occupied grid cells
  - no cell dominated solely by win rate
  - distinct representative builds per cell

- Archive is reproducible.

### Acceptance

- Search discovers builds that are behaviorally different and non-degenerate.

---

## Phase 11 — Integration and Feedback Loop

### Objective

Use search results to refine game rules and generation.

### Work

- Export best candidates as:
  - upgrade spec examples
  - balance hints
  - ArenaConfig fixtures

- Use discovered degenerate configurations to add validation rules.
- Feed descriptors back into:
  - WFC balance profile tweaks
  - AI transition priority tuning
  - ray instrument balance ratios

- Add regression tests for previously discovered broken configs.

### Tests

- Each discovered degenerate case becomes a permanent validation test.
- Existing map contract and physics tests remain green after tuning.

### Acceptance

- Automated search produces human-reviewable, mechanically meaningful
  configurations.

---

## Guiding Rules

- Determinism is non-negotiable.
- Physics bugs must be rejected, not evolved.
- Win rate alone is not a goal.
- Player and NPC actor assembly must share the same upgrade system.
- Actor/level search must never depend on graphical rendering.
- Every discovered degeneracy should become a test.

## Immediate Next Step

Begin with Phase 1. The first milestone is a `cargo test` proving:

```text
same ArenaConfig + same seed + fixed tick count
=> identical final snapshot and event log
```

Do not proceed to MAP-Elites until this is proven.
