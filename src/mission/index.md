# Spacetrash v11 Mission Plan

## 1. Project Mission

Spacetrash v11 is a continuous, real-time space salvage adventure built in Rust with Bevy. The player commands a crew, explores procedurally generated derelict spacecraft, and fights enemies while scavenging technology. Unlike a traditional roguelike, levels are not fixed: spaceships are individual room-graphs that can dock with one another to form one expanding playfield. The player's own ship serves as a base, but once docked it becomes a vulnerable boardable zone.

The game has no separate concept of a "level" or discrete mission: the playfield is always the current set of ships and their interconnections. Ships are authored as individual ship templates/blueprints using Eldiron, and scenarios define initial ship positions, docking states, and environmental conditions.

The game's design emphasizes emergent gameplay from simple, interdependent mechanics, inspired by Factorio but at a modest scale. A core distinguishing feature is the **live fog of war**: the player's view is determined by the current positions and capabilities of their agents, not by a static explored-map history. The game also draws inspiration from FTL: Faster Than Light; ship systems are interdependent machines that must be maintained, with cascading failures if neglected. Salvaging resources from these systems is a primary cause of those cascading failures, forcing the player to balance resource needs against ship stability. A hidden research/tech tree, presented diegetically as "research", gradually introduces mechanics in a manageable way.

## 2. Current Status

- **Engine**: Bevy 0.13 with default plugins.
- **Rust setup**: A working `cargo run` that opens a Bevy window.
- **What exists**:
  - Basic app skeleton (`src/app.rs`, `src/main.rs`).
  - `GameState` with startup transition from `Loading` to `MainMenu`, then to `Playing`.
  - `PhysicsConfig` and `InputConfig` resources.
  - `Velocity`, `Player`, `Ship`, `Room`, `DockingPort`, `AabbCollider`, `GridCoordinates` components.
  - A prototype `setup_scene` that spawns a camera, light, one 6x6 room (floor + four walls), and a movable player cube.
  - Simple input system (WASD/arrow keys) that applies acceleration and damping.
  - Simple movement integration and AABB collision resolution against static walls.
  - Camera follow system that keeps the camera at a fixed offset from the player.
  - Visible UI screens: loading screen, main menu, pause menu, game over screen, and in-game HUD.
  - Basic game loop: Loading -> MainMenu -> Playing -> Paused -> GameOver.
  - Input handling for starting the game, pausing/resuming, restarting, and quitting.
- **Known limitations**:
  - No circle colliders for actors; current AABB colliders are axis-aligned only.
  - No rotational/tank physics or angular velocity.
  - Collision resolution is simple and does not handle multiple simultaneous contacts well.
  - Only one room, no procedural ship generation.
  - No docking logic.
  - No multiple characters or AI.
  - No actor size system (small/standard/large).
  - No travel methods between ships (umbilicus, shuttles, teleporters).
  - No research/tech tree system.
  - Still lacking the typed room/tile graph representation.

## 2.5 Current TDD Progress

The project has adopted an unusually strong test-first development discipline.

All current tests are deterministic and run headlessly with `sh test.sh`.

### 2.5.1 Established Test Apparatus

- `src/testing/mod.rs` provides a deterministic headless `test_app()`.
- `TimePlugin` is disabled while `Time` is advanced manually at 60 Hz.
- Helpers exist for spawning actors, walls, key presses, and running ticks.
- `tests/` contains integration test targets.

### 2.5.2 Completed TDD Slices

- **Deterministic harness**
  - Fixed-timestep headless app is stable.
  - `run_n_ticks` advances time manually and deterministically.

- **Physics regression / collider extraction**
  - `src/systems/collision.rs` contains pure circle-vs-AABB and circle-pair resolution.
  - `CircleCollider` is used by dynamic actors.
  - `AabbCollider` remains for static walls.
  - `ActorSize` exists with `Small`, `Standard`, and `Large`.
  - `PhysicsBody` exists and is optionally used per-actor.
  - `movement_system` performs damping, integration, static-vs-dynamic resolution, and dynamic-vs-dynamic circle separation.

- **Layered constraints**
  - `src/procgen/layer.rs` derives:
    - `CellRequirement` for room system terminals.
    - `PathRequirement` for shared system layers between connected rooms.
  - Tests cover electrical paths between Reactor/Bridge, hydroponic terminals, and storage isolation.

- **Scaffold model**
  - `src/procgen/scaffold.rs` contains `RoomNode`, `RoomEdge`, `RoomGraph`, `RoomType`, and `SystemFlags`.
  - Tests cover room flags, graph add/neighbor relations, and connectivity.

- **WFC formalization**
  - `src/procgen/wfc.rs` supports:
    - local tile adjacency propagation.
    - deterministic greedy collapse.
    - seeded weighted collapse.
    - balance profiles with tile weight multipliers.
  - Tests cover deterministic identical runs, weighted selection, contradictions, and zero-weight rejection.

- **Graph Enhanced WFC stub**
  - `src/procgen/graph_enhanced_wfc.rs` exists.
  - The current solver is intentionally minimal and returns a valid full-grid placeholder.
  - The first test proves the GE-WFC interface can collapse a simple two-room graph without failing.
  - Graph-preserving door-cell assertions are deferred to the next GE-WFC implementation step.

### 2.5.3 Current Validation Coverage

Running `sh test.sh` currently executes:

- `actor_size` — actor-size radius mapping and collision filtering.
- `collision` — circle-vs-AABB and circle-pair pure tests.
- `gewfc` — GE-WFC interface smoke test.
- `layered_constraints` — graph-derived layer constraints.
- `movement` — input acceleration and overlapping actor separation.
- `scaffold` — room system flags, graph connectivity, and adjacency.
- `test_harness` — headless app initialization and actor spawning.
- `wfc` — local constraint propagation and determinism.
- `wfc_settings` — configurable seed and weighted balance profiles.

### 2.5.4 Immediate Next TDD Work

Priority order remains:

1. Make GE-WFC preserve actual door cells and room boundaries from the planar graph.
2. Bind room-type-specific tile sets and `WfcBalanceProfile` into GE-WFC.
3. Convert `spawn_ship_from_template` to consume the scaffold/GE-WFC output.
4. Add deterministic room interior tile generation with graph-derived system ports.
5. Add sensor-grid stub and test light/visibility propagation.
6. Add actor assembler using upgrade-driven components under the single ray-instrument rule.

## 3. Core Architecture Goals

The architecture follows the design in `src/meister/architecture.md` and the gameplay direction in `src/mien/design.md`:

- **ECS with Bevy**: Components for gameplay data, systems for logic, resources for global configuration and state.
- **Arcade physics**: Lightweight, continuous 2D gameplay physics (position, velocity, acceleration, damping, collision) rather than a full rigid-body simulator.
- **Procedural ship generation**: Each ship is generated from a planar room scaffold; WFC fills room interiors on that scaffold. Docking merges scaffolds into a single typed room/tile graph.
- **Dynamic docking**: Docking ports connect ship graphs into one large graph.
- **Multiple characters**: Player-controlled crew with switching, group commands, and AI for NPCs.
- **Roguelike elements**: Permanent death, limited resources, scavenging, upgrades.
- **Camera**: Isometric/top-down 3D camera that follows the selected character or party midpoint.
- **Rendering**: 3D visuals for a 2D gameplay plane.
- **Emergent gameplay**: Simple, interdependent mechanics combine to create depth. Mechanics are introduced gradually and interactions produce emergent tactical situations, not scripted set-pieces.
- **Ship systems as fragile machines**: Electrical, computer, fluidics, atmospherics, and thermal systems are simulated and can cascade into failure if not maintained; the bridge room provides maps for all systems. Salvaging resources from these systems is a primary cause of cascading failures, forcing the player to balance resource needs against ship stability.
- **Hidden research / tech tree**: A diegetically presented research system gradually unlocks mechanics, upgrades, room types, and systems, inspired by Factorio but hidden from direct view. Room types act as progression gates for ship systems.
- **Fluidics network**: Pipes, reservoirs, valves, and pumps allow moving fluids between rooms and ships via the docking umbilicus. Fluid transfer requires player control of all rooms along the path. Fluid types include oxygen, nitrogen, hydrogen, hydrazine, rocket fuel, coolant, water, and others.
- **Atmospherics simulation**: A low-frequency compartment model simulates room-level gas mixing, pressure, oxygen, and contamination, interacting with doors, breaches, life support, and salvage actions.
- **Actor size system**: Three sizes (small, standard, large) with differing collision radii and movement capabilities. Small drones can enter air vents and ignore collisions; standard actors require 1/3 tile; large actors require 2/3 tile.
- **Travel between ships**: Umbilicus airlock bridges (2-tile wide), shuttles (mini ships), and teleporters (expensive) provide limited inter-ship travel.
- **Live fog of war**: Visibility is driven by sensor grid propagation of current agent positions and sensors. There is no permanent explored-map state; areas are only visible while an agent/sensor actively updates them.
- **No discrete levels / continuous ship composition**: The active playfield is the union of all currently loaded ships, represented as a single typed room/tile graph.
- **First-class sensor grid and FSA**: Sensors use a shared cellular-automaton grid; runtime state is managed by finite-state automata.
- **Upgrade-driven actor generation**: All actors (player drones and NPCs) are assembled from upgrade modules; no hardcoded archetypes.
- **Automated arena playtesting**: Headless deterministic episodes with structured event logging and MAP-Elites search.

## 4. Immediate Next Steps (in order)

1. **Replace AABB player collider with CircleCollider**  
   - Add `CircleCollider { radius: f32 }`.
   - Use circle-vs-AABB and circle-vs-circle collision for movement.
   - Keep AABB colliders for static walls.

2. **Add angular velocity and rotation**  
   - Add `AngularVelocity` component.
   - Input system sets both linear and angular acceleration.
   - Movement system integrates rotation and handles orientation.

3. **Fake tank-style turning**  
   - Left/right turn inputs rotate the actor.
   - Forward/backward inputs set thrust.
   - Drift / momentum preserved from previous frames; damping reduces it.

4. **Broadphase collision**  
   - Implement simple spatial hash grid for dynamic entities to reduce collision checks.
   - Static walls remain in a separate query or grid for quick lookup.

5. **Collision events**  
   - Add a `CollisionEvent` resource or Bevy event.
   - Systems emit events on impact and resolution to drive gameplay (damage, sound, etc.).

6. **Implement planar room scaffold and typed room/tile graph**  
   - Add `RoomNode`, `RoomEdge`, `RoomType`, `SystemFlags`.
   - Generate a connected planar scaffold for ships.

7. **Add WFC interior stub**  
   - Fill room interiors with placeholder tiles, preserving door positions.

8. **Implement docking system**  
   - Add `DockingPort` entities at compatible scaffold edges.
   - Merge scaffolds into a single typed graph.

9. **Add shared sensor grid and `LightEmitter`/`LightReceiver`**  
   - Build cellular-automaton light/visibility propagation.
   - Enforce single ray instrument per agent.

10. **Multiple characters and basic AI**  
   - Spawn several crew members using the same upgrade system.
   - Add simple behavior FSAs and arbitration.

11. **Combat prototype**  
   - Add health, weapons, and simple projectiles/melee.

12. **Camera improvements**  
   - Smooth follow with lerp, rotation support.

13. **Implement actor size system**  
   - Add `ActorSize` component (`Small`, `Standard`, `Large`).
   - Adjust collider radii based on size: `Small` = no collision, `Standard` = 1/3 tile radius, `Large` = 2/3 tile radius.
   - Allow small actors to enter air vents (mark vent tiles/passages).

14. **Implement travel methods between ships**  
   - Add `MainAirlock` component and umbilicus bridge creation on docking.
   - Add shuttle spawning/docking logic.
   - Add teleporter pad components and teleportation system (with power/path costs).

15. **Implement hidden research / tech tree system**  
   - Add `ResearchState` resource and `ResearchStation` component.
   - Create a basic tech-tree graph with a few nodes.
   - Allow research projects to unlock upgrades, room types, and travel methods.

16. **Fluidics and Atmospherics integration**  
   - Later steps for fluidics/atmospherics simulation as described in architecture.

17. **Automated arena runner**  
   - Build headless deterministic episodes with event logging.
   - Connect MAP-Elites after baseline systems are stable.

## 5. Longer-term Roadmap

- Procedural derelict generation with WFC interior tiles.
- Ship-to-ship combat and flight.
- Command center capture mechanics.
- Save/load system.
- Asset pipeline.
- UI menus and settings.
- AI pathfinding across docked ship graphs.
- Economy and equipment.
- Ship system maintenance and cascade failure simulation.
- Salvage actions that damage specific ship systems, triggering cascades.
- Fluidics piping, reservoirs, and umbilicus transfer.
- Atmospherics gas-mixing compartment simulation.
- Actor size system full implementation (air vents, collision filtering).
- Travel methods: umbilicus bridges, shuttles, teleporters.
- Air vent navigation for small drones.
- Full hidden research/tech tree integration with Eldiron data.
- Eldiron-authored ship templates and scenario data loading.
- Procedural generation algorithm registry with loadable algorithms.
- First-person drone camera and visible-light-source logic.
- Automated playtesting and behavior search after baseline systems are stable.

## 6. Task Tracking

This file will be updated as milestones are reached.

## 7. Eldiron-Derived Technical Notes

Eldiron authors room scaffolds, system flag assignments, WFC tile sets, and
upgrade definitions. It does not author the runtime typed graph directly;
that graph is built by the generation pipeline from scaffold data.

## 8. Bevy-Derived Technical Notes

Bevy owns the runtime typed room/tile graph and all ECS state. It runs
headless deterministic episodes for automated testing.

## 9. Sensors, Dynamic Lighting, and In-Game Cameras

Sensors use the shared sensor grid; cameras are single ray instruments.
See `src/mission/rays.md`.

## 10. Rectangular Rooms, WFC Interiors, and Ray Interaction

Rooms are rectangular scaffold nodes; WFC interiors may add diagonal
occluders. The raycast backend still uses collider geometry for specific
queries, but most visibility is obtained from the sensor grid.

## 11. Actor Upgrades and Procedural Monsters

Actors are assembled from upgrade modules. Behavior upgrades contribute
finite-state automata states. Enemies are generated by sampling the same
upgrade pool.

## 12. Arcade Physics Formalization

See `src/mission/physics.md`. The physics layer remains unchanged, but
colliders now feed both physics and raycast queries.

## 13. Automated Arena Playtesting and Upgrade-Driven Search

After baseline physics, sensor grid, and scaffold generation are complete,
an automated arena runner will evaluate generated configurations.

Search uses MAP-Elites to find behaviorally distinct, non-degenerate builds.
See `src/mission/ai.md` and `src/mission/ecs.md` for details.
