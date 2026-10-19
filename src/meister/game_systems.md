# Game Systems and Emergent Mechanics Philosophy

## Status

Accepted design direction, pending implementation behind tests.

## 1. Philosophical Stance

Spacetrash v11 is not designed by predetermining a list of fun mechanics and then
building systems to create those mechanics.

Instead, the project follows this sequence:

1. Build a deterministic physical simulation and systemic simulation
   foundation.
2. Let each ship system have a clear, minimal physical/network representation.
3. Put balancing and layout variety into Graph-Enhanced WFC generation.
4. Use automated search and headless tests to discover which combinations of
   rooms, systems, filters, and behaviors produce interesting, non-degenerate
   play.

Our goal is **not** to hand-author fun.

Our goal is to create conditions under which fun configurations can emerge,
then use search to find and preserve them.

This means:

- Game mechanics are allowed to emerge from the interplay of systems.
- No system is privileged as the “main” mechanic.
- Interlocking failures, cascades, and difficult trade-offs are desired, but
  they must not be scripted.
- Balance knobs must not creep into physics, movement, AI, or UI.
- Balance lives in GE-WFC generation profiles, FSA transition tables, and
  global progression/research data only.

---

## 2. Separation of Concerns

The game is split into four clean domains:

### Physics

- Only answers: “Where can actors move, and how do actors collide?”
- Continuous arcade physics on the XZ plane.
- Circle colliders for actors.
- AABB/line colliders for static geometry.
- No system-specific logic, no statefulness about power, fluid, heat, or
  visibility.
- Physics is dumb on purpose.

### Ship Systems

- Each system is a graph, cellular automaton, or compartment model.
- Systems run independently and at different frequencies.
- Systems interact only through shared state resources and explicit events.
- No ship system directly mutates actor velocity or collider geometry.
  It may change door state, which indirectly changes collider availability.

### Sensors and Fog

- Sensors answer: “What information is currently available?”
- Shared sensor grid and light grid, cellular automaton propagation.
- One active ray instrument per actor.
- Fog of war decays without active sensor updates.
- Sensor output affects UI and AI, never physics.

### Generation and Balance

- Graph-Enhanced WFC owns all balance and layout-related randomization.
- Room types, system flags, door placement, conduits, pipes, vents,
  networks, ports, and starting system states are determined here.
- The same generated world can be reproduced exactly from a seed.
- Balance knobs are explicit fields in `GraphWfcBalanceProfile` and
  generation configuration, never hidden inside simulation code.

---

## 3. Layer-by-Layer System Vision

Each ship layer has a distinct simulation resolution, gameplay identity,
and GE-WFC balance surface.

### 3.1 Structural

- **Model:** tile graph + room graph.
- **Resolution:** tile + room, with door FSAs.
- **Simulation style:** static structure, door/open state updates.
- **Gameplay:** movement, cover, chokepoints, room identity, door control.
- **GE-WFC balance:** room sizes, door placement, deadzones, interior tile
  density, corridor width.

Physics remains the only system that reads structural colliders.

### 3.2 Electrical

- **Model:** graph edges + tile-level conduction.
- **Resolution:** edge + tile; early ships use room-level propagation,
  advanced ships use wire tiles.
- **Simulation style:** network flow + FSA.
- **Gameplay:** power routing, breaker control, overloads, salvaging live
  power systems, cutting power to enemies.
- **GE-WFC balance:** generator placement, wire density, breaker state,
  pre-existing faults, room-type gating.

Electrical state is a property of edges and terminal rooms. It does not
change actor movement.

### 3.3 Fluidics

- **Model:** directed pipe networks with room reservoirs.
- **Resolution:** edge + tile; valves, pumps, tanks.
- **Simulation style:** network flow with pressure/volume.
- **Gameplay:** transferring fluid between rooms and ships, securing theft
  paths, causing leaks, contaminating supplies, extinguishing fires with
  water.
- **GE-WFC balance:** pipe density, valve placement, reservoir starting
  volume, leak probability, fluid type distribution.

Fluidics affects atmospherics, thermal, and hazards, not raw physics.

### 3.4 Atmospherics

- **Model:** room-level compartment gas simulation.
- **Resolution:** room bulk mixing; edges are doors, vents, breaches.
- **Simulation style:** low-frequency O(rooms + edges) compartments.
- **Gameplay:** pressure management, oxygen management, suffocation,
  venting rooms, life-support sabotage, seal-and-drain tactics.
- **GE-WFC balance:** vent placement, breach probability, room volume
  distribution, starting gas mixtures.

Atmospherics modifies room FSA states, hazard visibility, and fire behavior.

### 3.5 Thermal

- **Model:** room heat + tile fire CA.
- **Resolution:** room for bulk heat, tile for fire.
- **Simulation style:** compartment heat transfer + cellular automaton.
- **Gameplay:** overheating equipment, starting or extinguishing fires,
  using fire as a weapon/diversion, cooling reactor rooms.
- **GE-WFC balance:** insulation per room, radiator placement, flammable
  interior density, starting heat/fire probability.

Thermal damages actors through system events, not through physics collisions.

### 3.6 Computer / Network

- **Model:** graph nodes and edges.
- **Resolution:** graph; nodes are terminals, cameras, controllers.
- **Simulation style:** graph FSA and access control.
- **Gameplay:** hacking doors, disabling sensors, overriding life support,
  injecting viruses, protecting your own network.
- **GE-WFC balance:** network cable density, access terminal placement,
  firewall strength, remote-controllable door ratio.

Computer state changes door/sensor/system states, never collider logic.

### 3.7 Docking / Travel

- **Model:** structural merge of two ship graphs.
- **Resolution:** edge; docking port graph nodes.
- **Simulation style:** graph merge + system compatibility checks.
- **Gameplay:** boarding actions, transferring only shared systems,
  defending airlocks, cutting umbilicus.
- **GE-WFC balance:** port placement, system compatibility, airlock bridge
  length, docking security.

Docking creates new walkable structure and system edges; physics merely
sees new colliders after merge.

### 3.8 Sensors / Fog of War

- **Model:** per-tile cellular automaton overlay.
- **Resolution:** tile CA.
- **Simulation style:** propagation and decay.
- **Gameplay:** managing limited sensor packages, choosing ray instruments,
  leaving relays, using light and non-visible sensors.
- **GE-WFC balance:** occluder density, built-in lights, sensor terminal
  placement.

Visibility affects AI and UI only. It never moves actors.

---

## 4. Complexity Introduction and Room-Type Gating

Complexity is intentionally staged through room types.

| Room Type      | Primary System Introduced        |
|----------------|----------------------------------|
| Engineering    | Basic electrical                 |
| Reactor        | Electrical + thermal             |
| LifeSupport    | Atmospherics                     |
| Hydroponics    | Fluidics + atmospherics          |
| ComputerCore   | Computer/network                 |
| Bridge         | Ship-wide system maps and control|

A newly generated ship does not need to contain all systems. The presence or
absence of a room type controls which systems are locally active.

This parcels complexity gradually and keeps physics unaffected.

---

## 5. Implementation Order

Systems are built test-first in this order:

1. Structural + door FSAs.
2. Electrical room-level graph and breaker/door interaction.
3. Sensor grid + fog of war.
4. Atmospherics room-level gas mixing.
5. Fluidics graph network and umbilicus transfer.
6. Thermal room heat and simple fire CA.
7. Computer network and hacking.
8. Docking graph merge and shared-system compatibility.

Each system begins as a pure graph/CA/compartment model with unit tests.
Only after the pure model is stable does it get integrated into Bevy and
visuals.

No system-specific balance value is allowed inside physics or movement.

All balance lives in GE-WFC generation profiles, FSA transition tables, and
global progression data.

---

## 6. Long-Term Goal

The final game emerges from the interaction of independently modeled
systems, not from a predetermined list of hand-tuned mechanics.

The majority of long-term development is not writing “features”.

It is:

- Building clean, deterministic system simulations.
- Exposing balance surfaces through GE-WFC.
- Running automated arena search.
- Observing which combinations produce meaningful decisions, cascades,
  risky salvage, and distinct drone/upgrade behavior.
- Keeping those results while rejecting degenerate or broken configurations.

This is the core philosophy of Spacetrash v11: **simulate, search, and let
the fun emerge.**
