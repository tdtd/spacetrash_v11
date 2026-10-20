# Spacetrash v11 Design Document

## 1. High Concept

**Spacetrash v11** is a top-down 3D-rendered roguelike-like adventure set in
the debris fields of a forgotten star system. The player commands a small
salvage ship and a crew of multiple characters, exploring derelict stations,
scavenging technology, and surviving the dangers of deep space.

Unlike traditional roguelikes, movement and combat are continuous, not grid
or turn-based. Characters move freely in the world, and action happens in
real time.

A unique element is that **levels are not fixed**. The world is composed of
individual spaceships, each with its own procedurally generated grid of
rectangular rooms. When ships dock with one another, they merge into one
large connected playfield. The player’s ship serves as a home base, but
while docked it is also a combat zone that enemies can board. Commandeering
an enemy ship means physically reaching its command center and taking it
over.

There are no discrete levels or missions; the world is the continuous
operation of the player's fleet and discovered ships. Each ship is a
self-contained grid, and the composition of ships at any time defines the
current playfield. Ships can be procedurally generated, manually edited, or
both. Scenario creation defines initial ship positions, docking states, and
environmental conditions using Eldiron.

## 2. Core Experience

Players will:

- Fly a small ship between floating station wreckage.
- Dock with derelicts and send crew members inside.
- Control multiple characters, either one at a time or as a group.
- Scavenge valuable tech, fuel, and supplies.
- Fight hostile drones, alien scavengers, and environmental hazards.
- Upgrade the ship and crew equipment.
- Perform research to unlock new mechanics and capabilities over time.
- Make decisions that affect survivability and story.

## 3. Game Pillars

1. **Continuous Movement & Physics**  
   Everything moves smoothly with simple momentum, collisions, and physical
   presence. There is no fixed grid for movement.

2. **Multiple Characters / Crew**  
   The player controls several characters, each with their own inventory,
   stats, and abilities.

3. **Scavenging & Economy**  
   The main loop revolves around finding salvage, returning to the ship,
   and using resources to improve equipment and ship systems.

4. **Roguelike Progression & Danger**  
   Death is permanent for characters; resources are limited; each expedition
   carries risk.

5. **Dynamic Ship Docking**  
   Ships can dock together, creating a single larger map. This makes the
   playfield change over time and allows boarding actions.

6. **Emergent Complexity from Simple Mechanics**  
   The game is inspired by games like **Factorio**, where a small set of
   simple, interdependent mechanics are introduced gradually and their
   interactions produce deep, emergent gameplay. In Spacetrash, we apply the
   same principle at a deliberately modest scale. The interplay between
   modular upgrades, sensor limitations, resource scarcity, room layouts, and
   crew management creates emergent tactical situations instead of scripted
   set-pieces. Unlike Factorio’s focus on large-scale automation, Spacetrash
   focuses on small-unit tactics and salvage operations, but the emergence
   remains central.

7. **Hidden Tech Tree / Research**  
   Spacetrash includes a hidden tech tree, inspired by Factorio’s technology
   progression. Unlike Factorio where the tech tree is visible in a
   dedicated UI, here it is represented diegetically as **research**. The
   player interacts with research stations or computer terminals to perform
   "research", which gradually unlocks new mechanics, upgrades, room types,
   ship systems, and equipment. The tech tree is not directly visible; its
   contents are revealed procedurally based on player actions, salvage
   findings, and environmental needs. This design principle allows the game
   to introduce complexity gradually, ensuring that new mechanics appear only
   when the player is likely ready and not overwhelmed.

## 4. Gameplay Loop

1. **Embark**  
   Choose a derelict target and fly the ship to it.

2. **Dock**  
   Connect your ship to the derelict or another ship. The two ships merge
   into one continuous playfield with a shared grid of rooms.

3. **Explore**  
   Send crew inside, move through interconnected rooms and corridors, and
   uncover the layout.

4. **Scavenge**  
   Search containers, hack terminals, mine wreckage, and collect loot.

5. **Combat**  
   Fight enemies using real-time continuous movement and aiming. Enemy
   boarders may enter your ship through the docking connection.

6. **Research**  
   While exploring, discover research nodes or salvage data fragments that
   enable new technologies. Research is performed at dedicated stations or
   through portable field kits, unlocking mechanics as needed.

7. **Commandeer**  
   If the enemy ship has a command room, fight your way to it and take
   control. The ship then becomes a permanent part of your fleet or base.

8. **Return**  
   Get back to your ship before fuel/oxygen runs out or the derelict becomes
   too unstable.

9. **Upgrade**  
   Spend salvage and research points to improve crew gear, ship modules, and
   unlock new areas or systems.

10. **Repeat**  
    The star system is persistent; explored ships remain explored, but new
    wrecks can be discovered and docked later.

## 5. World

- The game takes place in a 3D-rendered star system.
- Locations are floating stations, wreckage clusters, asteroid belts, and
  the player’s small ship interior.
- The view is top-down or isometric, but the world is fully 3D.
- Terrain is not grid-based for physical collision; instead, each ship is a
  grid of rectangular rooms connected by corridors.
- Ships are the fundamental units of the playfield. There is no separate
  "level" concept; the active world is the composition of all currently
  loaded ships, which may be procedurally generated, manually edited in
  Eldiron, or a combination of both.

### Ship Grids

- Every ship is an independent grid of rectangular rooms.
- Rooms vary in width and height.
- Every room is surrounded by a **1-tile deadzone**. This deadzone is a clear
  perimeter around each room, and the gap between adjacent room outer walls
  is exactly one tile. No gameplay actor can stand inside the deadzone except
  while passing through a door.
- Adjacent rooms are connected by **1-tile-wide doors** placed on shared
  axis-aligned edges. The door opening spans exactly that 1-tile deadzone.
- Doors or open passages connect adjacent rooms through the deadzone.
- Ships are generated procedurally from seed data, and can be further
  refined through manual edits in Eldiron.
- Eldiron supplies ship templates, room templates, and scenario data that
  define initial ship composition, positions, and docking states.

Procedural generation is now a first-class concept. A `ShipGenerationAlgorithm`
trait and a `ProcGenRegistry` resource define the interface for pluggable
generation algorithms. These algorithms can be written in Rust, loaded from
code, and later adapted from scripting languages such as Lua, Rhai, or WASM.
The same algorithm may produce a ship or a whole scenario, and the result can
always be further tweaked in the editor.

### WFC Interior Tiles

- The generation grid of rooms remains rectangular and axis-aligned.
- Wave Function Collapse (WFC) is used to populate the **interior** of each
  room with complex grid tiles, not to define room boundaries.
- Interior tiles may include diagonal walls, low cover, consoles, storage,
  hazards, and decorative props.
- These interior tiles may contain diagonal occluders, but they are limited
  to the inside of the rectangular room.
- Room connectivity, door placement, and ship graph traversal are unaffected
  by the interior tile details.
- The aesthetic remains “grid-ish”: the world is still authored on a grid,
  but room interiors can contain angled or non-rectangular tile features.
- WFC tile definitions include edge labels, rotation variants, weights, and
  room-type tags for interior placement rules.

### Room and Resource Variety

The original Duskers featured only a small number of item types and special
rooms. Spacetrash deliberately expands this space with many new room types
and resources. Examples include:

- Room types: engineering, medical, storage, command, hydroponics, security,
  reactor, cargo hold, crew quarters, and more.
- Resources: fuel, oxygen, scrap metal, rare minerals, data cores, medical
  supplies, weapon parts, upgrade modules, and more.

These additions are not cosmetic. Each room type and resource can interact
with the existing systems (sensors, upgrades, docking, physics) to create new
emergent challenges and opportunities. The variety is designed to feed into
the emergent gameplay loop rather than simply providing more loot.

### Ship Systems and the Bridge

In addition to room and resource variety, each ship simulates several interdependent systems:

- **Electrical:** power distribution, wiring, batteries, reactors.
- **Computer:** navigation, sensors, automated doors, security terminals.
- **Fluidics:** fuel lines, coolant, sewage, water reclamation.
- **Atmospherics:** oxygen generation, scrubbers, pressure regulation.
- **Thermals:** heat management, radiators, insulation, fire suppression.

These systems are not cosmetic. They interact with each other and with room states. For example, an electrical failure can disable lighting and doors, reduce oxygen generation, or cause a thermal radiator to overload. An atmosphere leak can trigger low-pressure hazards. Failures cascade: a small problem in one system can cause secondary failures elsewhere, forcing the player to prioritize repairs, manage limited resources, and sometimes abandon a section of the ship.

The **bridge** room is particularly important: from the bridge, the player can access the ship-wide system maps for all these systems. This replaces the need to visit each individual room to diagnose problems. Without the bridge, the player must rely on local sensor readings, visual inspection, and educated guesses. Capturing or reaching a bridge therefore becomes a strategic objective.

This system design adds a layer of tension inspired by FTL, but scaled to the small-unit salvage context of Spacetrash.

### Fluidics and Umbilical Transfer

Spacetrash extends Duskers with a full fluidics system. Ships contain reservoirs of various fluids and gases required by their systems. These fluids can be moved through a network of pipes that traverse rooms. The player can use pumps and valves to transfer fluid between reservoirs, including between ships through the docking umbilicus, provided the player controls all rooms along the route between the two points. This control requirement ensures that fluid theft is not trivial; the path must be physically secured or the transfer risks sabotage, pressure loss, or backflow.

Fluid ingredients include, but are not limited to:

- Oxygen (for atmospherics and life support)
- Nitrogen (for atmosphere inerting and pressure)
- Hydrogen (for fuel cells and chemical processes)
- Hydrazine (monopropellant for thrusters)
- Rocket fuel (liquid or gel propellant)
- Coolant (thermal control)
- Water (subsystem maintenance, drinking)
- CO₂ scrubber chemicals
- Lubricants
- Waste fluids (to be ejected or processed)

These fluids are not just resource counters; they occupy volume in reservoirs and must be transported through pipes with limited flow rates. Overpressurization, contamination, leaks, and freezing can occur based on environmental conditions and system damage. The bridge map shows a simplified fluid schematic, but local pipe gauges and valve handles provide diegetic control.

Fluid theft through the umbilicus is a core strategy: dock with a derelict, secure the route from a valuable reservoir to your ship, and then pump it. However, the act of pumping may destabilize the derelict’s fluidics, causing cascading failures in other systems as described earlier. The player must weigh the value of stolen fluids against the risk of accelerating the derelict’s destruction.

### Visible System Lines and Manual Tracing

Ships are not just abstract system schematics; their physical systems have visible in-world representations:

- Electrical wiring appears as thin conduit lines running along ceilings.
- Fluid piping appears as thicker lines running along floors.
- Atmospheric ducting may appear as square ducts near walls.

These visible lines are important for exploration and sabotage:

- A drone with a camera can see them directly, even without accessing the ship’s computer.
- The schematic map can later overlay the same data when the bridge/computer layer is active and enough network data has been gathered.
- If map data is unavailable, a player can send a drone physically into a room and manually trace a wire or pipe by following its visible path back to a source.
- Manual tracing is slow, dangerous, and subject to fog of war, but it works without any computer or bridge access.

WFC generation places these system entities in plausible locations:
- Conduits and pipes appear more often in rooms that have the corresponding
  system flags.
- The exact tile positions are still determined locally, so every ship has
  a unique physical system layout while respecting the room graph and system
  progression gates.

### Atmospherics Gas Mixing Model Plan

Unlike the original Duskers, which modeled only room-by-room vacuum and radiation, Spacetrash will simulate a more complex atmosphere. The performance-friendly approach is a **compartment model**:

- Each room is a single, well-mixed volume with uniform pressure, temperature, and gas mixture.
- Track multiple gas species: oxygen, nitrogen, carbon dioxide, contaminants, and later additional species as needed.
- Room volume is derived from the room’s rectangular dimensions.
- Connections between rooms are represented as `AtmosphereEdge` entities, including open doors, vents, pipes, and hull breaches.
- For hull breaches, a special `SPACE` node at vacuum is connected to the room.
- A low-frequency simulation tick (e.g., 1 Hz) updates each room:
  - Bulk flow between adjacent rooms from pressure differences.
  - Diffusive mixing from concentration differences, even without pressure gradients.
- The simulation cost is `O(rooms + edges)` per tick, easily within the prototype’s 50-room budget.
- Visual effects such as smoke, escaping gas, and fire use particles or billboards driven by the compartment model, not a full fluid simulation.
- If later more spatial detail is needed, we can add a hybrid model with sub-room diffusion only for active rooms where leaks or fires are present.

This model interacts with:
- Life support systems adding oxygen and removing CO₂.
- Salvage actions that siphon oxygen or breach hulls.
- Sensors that report room pressure, oxygen percentage, and contamination.
- Thermal system adding heat, which can increase pressure or trigger further breaches.
- Door state changes that open or close `AtmosphereEdge` connections.
- Bridge system maps that display room atmospheric status at a glance.

### Salvage as a Catalyst for Cascading Failure

The player’s resource needs are pressing: fuel, oxygen, scrap, rare minerals, data cores, and spare parts are all required for repairs, upgrades, and survival. However, salvaging these resources regularly means extracting components that are still part of a ship’s operational systems. This creates a fundamental tension: the most valuable salvage is often exactly what keeps the derelict stable and habitable.

Examples of salvage-driven cascades include:

- **Removing the warp core** eliminates the ship’s primary power source. Without that power, doors stop, lights go dark, life support begins to fail, and automated defenses may become uncontrolled.
- **Disabling the computer and extracting the radiator** may allow the player to claim valuable processing and cooling hardware, but it also removes the coordination and heat-dissipation layers. Thermal buildup in the reactor can then trigger overload warnings, fires, or even reactor explosion.
- **Siphoning away the oxygen reserves** for portable air tanks may give the crew extra breathing time, but it lowers the ship’s atmospheric reserve. Atmosphere scrubbers cannot compensate, leading to low-pressure hazards, crew asphyxiation risk, and sealing doors.
- **Draining coolant or fuel lines** to refuel the player’s own ship may keep the derelict running for a while, but eventually thermal or fluidic imbalances cascade into electrical short circuits or pressure failures.

Because the player’s own ship requires many of the same resources, there is constant temptation to strip a derelict to the bone. Each extraction is a risk assessment: how much can be taken before the derelict becomes a death trap? The system interactions turn every salvage operation into a puzzle with delayed consequences, and they ensure that the player never sees a ship as a static loot container.

This implies that a good captain must balance resource acquisition against the stability of the environment they are standing in. Sometimes the optimal play is to take only what is needed and leave the rest, while other times a controlled cascade can be used as a weapon against enemies. The live fog of war amplifies this tension: you may not see the consequences until it is too late.

### Docking Maps

- When two ships dock, their grids are linked at the docking ports.
- The combined world is one large playfield: all rooms are reachable.
- The player’s ship remains a base even while docked, but enemies can now
  board it through the docking port.
- The same docking mechanism can later be used to connect multiple ships
  into a sprawling station.

### Hybrid Docking Model

Spacetrash intentionally departs from Duskers by making docked ships share a single continuous interior map. However, we do not simulate ships flying around in 3D space. Instead, docking is handled with three distinct layers:

1. **Space navigation graph** — FTL-like abstract graph of ships, stations, shuttles, and dock links. Travel between distant objects is a deliberate command, not continuous flight.
2. **Interior shared map** — after docking, the involved ships and station become one merged `RoomGraph` and tile grid. Continuous XZ movement, colliders, doors, fog of war, and system lines operate on this shared map.
3. **Docking FSA + merge transaction** — docking/undocking is a deterministic structural state machine that creates or removes an umbilical corridor and updates the world graph, tile grid, colliders, and system graphs atomically.

The result is a hybrid experience:

- Fleet/ship management stays as manageable as FTL.
- Boarding gameplay stays as continuous as the current Spacetrash prototype.
- Docking becomes a real strategic action, not just a teleport.

#### Docking State Machine

A docking port moves through clear states:

`Separated -> Aligned -> Clamped -> Sealed -> Powered -> Pressurized -> Open`

Failure and danger states include:

`Rejected`, `Misaligned`, `Jammed`, `Unpowered`, `Breached`, `Decompressing`

This creates gameplay around power, atmosphere, compatibility, forced undocking, and station lockdown controls.

#### Umbilical

Between connected main airlocks, a two-tile-wide passable corridor is generated. The corridor is a `RoomEdge` of kind `Umbilical`, and its tile geometry is added to the merged grid. It can stretch within validated bounds and may be enclosed by wall or hazard shells. The same corridor is used for shared-system transfer.

#### Four-Sided Station

The station is generated as a central hub with one main airlock per side. It is not merely another ship. It provides dock control, station-wide system mapping, and central chokepoints. The completed map forms one connected `RoomGraph`.

#### UI and Testing Implications

- Editor tools must allow placing/moving ships and snapping airlocks visually.
- Schematic layers must show docking state, umbilical routes, and system compatibility.
- HUD/terminal commands must expose docking actions and statuses.
- Map contracts must validate connectivity, overlap avoidance, two-tile umbilical width, and system graph pruning across docking edges.

### Travel Between Ships

There are only a few limited means of travel between ships:

- **Umbilicus / Airlock Bridge**  
  Ships with a main airlock can extend a **2-tile-wide bridge** that connects to another main airlock. This forms a stable, protected corridor that allows actors to move between the two ships. The umbilicus is also the route used for fluid transfer, as described earlier.

- **Shuttles**  
  Shuttles are mini one-room ships that can be parked next to any open door. They act as temporary connections between ships or between a ship and the player’s base. A shuttle is essentially a small ship with its own atmospheric state, but it can dock at any open door, providing a mobile airlock.

- **Teleporters**  
  Teleporters are a highly expensive and rare technology. They allow instantaneous travel between two teleporter pads within a limited range. Teleportation consumes significant power and may have restrictions, such as requiring line-of-sight or a stable power grid. They are not a common solution but an end-game or high-risk option.

These travel methods are modular and can be extended later.

## 6. Characters

Characters are crew members with:

- **Stats:** Health, stamina, carry weight, speed, radiation resistance.
- **Inventory:** Weapons, tools, consumables.
- **Abilities:** Active skills and passive perks.
- **Equipment:** Armor, weapon, backpack, tool.

Players can switch between characters and also give group movement/action
commands. AI companions will follow, guard, or perform assigned tasks.

## 7. Movement & Physics

The game uses a lightweight **arcade physics** model, not a full rigid-body
simulator. Physics operates on the XZ plane (Y is up for rendering only).

### Core Principles

- **Continuous movement** with velocity and acceleration.
- **Arcade feel:** responsive, predictable, and forgiving.
- **Deterministic** fixed timestep integration.
- **Circle colliders** for actors, **AABB/segment colliders** for static
  geometry.
- **Fake physics** where needed: drift, knockback, and push effects are
  implemented as direct velocity changes, not true impulse-based responses.

### Input and Control

- WASD/arrow keys control movement.
- Left/Right turn rotates the actor (tank-style), while Forward/Backward
  applies thrust along the actor's facing direction.
- Alternative mouse-driven movement may be added later; both will map to a
  common `DesiredMovement` resource.

### Movement Model

- Each actor has:
  - `Velocity` (XZ vector)
  - `AngularVelocity` (yaw rate)
  - `Rotation` (yaw)

- Input sets angular acceleration and forward thrust.
- Damping reduces both linear and angular velocity over time.
- Max linear speed is clamped.
- Drift can be simulated by only partially rotating the velocity vector into
  the new facing direction, giving a tank-like sliding feel.

### Collision

- Dynamic actors use `CircleCollider`.
- Static walls use `AabbCollider` (axis-aligned boxes).
- Future interior diagonal walls will use `LineSegmentCollider`.
- Broadphase spatial hash avoids O(n²) checks.
- Collision resolution pushes the circle out of static AABBs/segments along
  the minimum penetration axis.
- Dynamic-dynamic collisions are resolved with positional correction and
  optional velocity adjustments.

### Actor Sizes and Movement

- There are three actor sizes: **Small**, **Standard**, and **Large**.
- **Small** actors (e.g., tiny drones) can navigate through air vents and do not collide. They can pass through other actors freely.
- **Standard** actors require **one-third of a tile** of space and use a collider radius of approximately `0.1667` tiles. Two standard-sized actors can pass each other in a hallway or door.
- **Large** actors require **two-thirds of a tile** of space and use a collider radius of approximately `0.3333` tiles. A single large actor cannot pass a standard or large actor in a 1-tile-wide door or hallway.
- The size of an actor is determined by an `ActorSize` component; collider radii are set accordingly at spawn time based on the size.

This size system affects movement, docking, and combat (e.g., large actors cannot use air vents and may block corridors).

### Physics and Upgrades

- Movement upgrades define physical parameters:
  - Wheel drive: high friction, lower turn rate
  - Legs: medium friction, better turning
  - Hover: low friction, sliding

- `PhysicsBody` component stores mass, friction, damping, and restitution.
- AI enemies share the same physics components, so emergent behaviors can
  include movement differences.

## 8. Combat

- Real-time aiming with mouse or gamepad.
- Weapons have fire rate, spread, projectile speed (if applicable), and
  damage falloff.
- Melee attacks have range, arc, and swing time.
- Characters can take damage, and health regenerates slowly or via items.
- Cover and line-of-sight matter, including occlusion from interior diagonal
  tiles inside rooms.

## 9. Camera & View

- Default camera is isometric top-down, giving depth and a 3D look.
- The camera follows the selected character or the midpoint of the party.
- The player can rotate the camera 90 degrees for viewing behind obstacles.
- There is exactly **one continuous visual feed per player**.
- The continuous feed is produced by a real in-game camera entity. It may be
  the main viewport, a helmet cam, a security camera, or a drone camera.
- The player can switch which camera is the active continuous feed, but only
  one feed is live at any time.

### First-Person Drone Perspective

The focused drone or Blackbox provides the player’s point of view when the
first-person camera mode is active:

- Number keys `1`–`9` switch focus to individual drones.
- Number key `0` focuses the Blackbox.
- The active world camera is placed at the focused entity’s eye offset and
  adopts the entity’s facing.
- The first-person world camera is separate from the schematic map overlay.
  Pressing backtick or changing schematic layers must not move or rotate the
  first-person camera.

This separation is core to the feel of the game: the player sees through the
drone’s sensors, while the schematic map remains a diegetic command overlay.

### Live Fog of War

Spacetrash does **not** use a traditional minimap that permanently reveals
explored areas. Instead, the player’s view is a **live fog of war** driven by
the current positions and capabilities of their agents. What is visible at any
moment depends on the active cameras, lights, and sensors carried by the crew.
Areas are not simply “uncovered”; they are only visible while an agent (or a
remote camera) is present and functioning. When you leave a room, you lose
perceptual detail, and unless a sensor or camera is left behind, the room
becomes dark again. This creates a constant need for exploration, sensor
management, and careful positioning of the crew.

### Visible-Light Cameras Require Light

- A visible-spectrum camera cannot produce a usable image in complete darkness.
- It needs a light source in the visible spectrum, provided either by the same
  actor carrying the camera or by another nearby actor/emitter.
- Non-visible cameras, such as infrared or thermal, have different light and
  occlusion requirements.
- Interior diagonal tiles are occluders for camera and light rays, but room
  boundary walls remain axis-aligned.

## 10. UI and Diegetic Sensor Interfaces

- HUD shows selected character health, inventory, weapon, and quick slots.
- Sensor data is presented through diegetic interfaces rather than magical
  full-map vision.
- Radar is a sweeping line or expanding ping ring, not a live full map.
- Thermal sensors are a slow, noisy overlay or a periodic snapshot.
- X-ray/backscatter sensors produce short-range outline blips through thin
  walls, with heavy power or cooldown costs.
- Camera feeds are the only true real-time visual stream, and only one can be
  displayed at a time.
- Sensor cooldowns, sweep speeds, and ranges are visible to the player so the
  performance compromises read as intentional sci-fi equipment behavior.

### Traditional GUI vs Diegetic Console

Spacetrash deliberately splits interaction surfaces into two zones:

- **Traditional GUI** is used for the main path: main menu, scenario
  selection, pause menu, game-over screen, and basic mouse-driven navigation.
  These screens are outside the active drone/ship fiction and should be
  clear, discoverable, and conventional.
- **Diegetic command line** is used for debug and out-of-game commands once
  the player is inside the live world. Commands such as simulation speed
  control, scenario listing/reloading, and developer diagnostics live here,
  not in the HUD.
- **Debug information is kept out of the HUD.** The HUD shows gameplay-
  relevant state such as health, oxygen, selected unit, sensor view, and
  schematic layer. Console output and diagnostics belong to the terminal
  view.
- **Spectator mode is not a separate player-facing mode.** It is superseded
  by godmode/editor for interactive editing and by headless `run_arena` for
  deterministic replay. The game therefore has three developer-facing modes:
  live play, editor/godmode, and headless test.

### Schematic Layer Visual Language

The schematic map is intentionally coarse and diegetic. It does not show
real-time wall angles, drone rotations, or fine geometric details. It borrows
the visual language of roguelike terminals and engineering diagrams:

- **Structural layer:** Uses ASCII-style room outlines and simple hatch lines.
  Player bots are represented as ASCII glyphs rather than precise 3D models.
  Wall details, such as angled sections, are not shown; only room extents and
  rough connectivity are available.
- **Electrical layer:** Uses one distinct line-drawing style, such as thin
  cyan traces with small node markers, to represent power distribution.
- **Fluidics layer:** Uses a second distinct line-drawing style, such as
  green/blue dashed lines with valve glyphs, to represent piping and reservoirs.
- **Thermal layer:** Uses a heatmap overlay where room background color shifts
  from cool to hot instead of drawing individual temperature sensors.
- **Atmospherics layer:** Highlights ventilation systems and air paths, using
  icons or flow marks for vents, scrubbers, and pressure boundaries.
- **Other layers (computer, research, docking, sensors):** Use their own
  minimal glyph/line conventions, always keeping the roguelike-inspired,
  low-resolution feel of the interface.

The live fog of war is reflected in the UI: the minimap or tactical overlay
shows only what current sensors actually detect, not a persistent history of
visited rooms. This reinforces the need to position scouts, cameras, and
lights deliberately.

### Diegetic Command Interface and Keyboard Controls

The player interacts with the world through a diegetic command line and a
schematic map. This interface is keyboard‑driven and uses a small set of
reserved keys:

- **Number keys 1‑9** focus the view and control on one of the player’s drone
  units. Pressing a number immediately selects that drone and centers the
  camera on it, while the command line and schematic layers remain available.
- **Number key 0** focuses the view and control on the player’s main
  character, the “Blackbox”. This is the primary command unit for the player.
- **Backtick (`` ` ``)** toggles between the command line input and the
  schematic map. The command line is a text‑based diegetic interface for
  issuing orders and querying drone status, while the schematic map is a
  visual layer‑based representation of the ship systems and structure.
- **Function keys F1‑F12** toggle the schematic map layers. Each layer
  corresponds to a distinct aspect of the ship and world:
  - **F1** – Structural layer (rooms, walls, doors, docking ports)
  - **F2** – Electrical layer (power distribution, wiring, outlets, batteries)
  - **F3** – Fluidics layer (pipes, reservoirs, valves, pumps)
  - **F4** – Atmospherics layer (oxygen, pressure, contamination, vents)
  - **F5** – Thermal layer (heat, radiators, insulation, fires)
  - **F6** – Computer/network layer (terminals, sensors, cameras, data links)
  - **F7** – Research/Tech layer (research stations, tech nodes, progress)
  - **F8** – Docking/Travel layer (airlocks, shuttles, teleporters, umbilicus)
  - **F9** – Sensors/Tactical overlay (radar sweep, thermal snapshots, live
    fog of war)
  - **F10, F11, F12** are reserved for future or custom layers.

> **Prototype note:** during early prototypes, `F1` may temporarily toggle
> god mode. The final control scheme reserves the function keys for schematic
> layers and will use a separate key (e.g., `E` or a menu command) for editor
> mode.

These keyboard mappings are fixed and reserved for core gameplay. Future
mods or features must not remap them.

## 11. Audio

- Ambient space hum, distant explosions, station creaks.
- Weapon sounds, footsteps, alarms.
- Dynamic music based on danger level.

## 12. Win/Loss Conditions

- There is no single endgame; surviving and improving is the goal.
- A run can end if all crew members die.
- The player may restart with a new ship/crew, keeping some meta-progression.

## 13. Visual Style

- Low-poly or stylized 3D graphics.
- Debris fields, flickering lights, rusted metal, sci-fi salvage.
- Contrast between dark space and bright salvage glints.
- Grid-ish layouts remain visible through ship interiors, but WFC interior
  tiles break the monotony of pure rectangles without changing room
  boundaries.

## 14. Current Prototype Scope

For the first playable prototype:

- One small base ship and one derelict ship to dock with.
- Both ships generated procedurally with rectangular room grids.
- A docking port that joins them into a single playable map.
- Two or three crew members with basic movement and one weapon each.
- Simple continuous physics and collision against walls.
- Basic loot containers and an enemy drone.
- Ship as a safe zone until docking; after docking, enemies can board.
- Exactly one continuous camera feed that the player can control.
- A small number of periodic/sweeping sensors such as a radar ping and a
  thermal scan.
- Visible light and shadow affect whether the active visible camera can see
  clearly.
- No multiplayer yet, but the architecture must reserve one continuous feed
  per future player.
- Initial generation focuses on axis-aligned rooms with 1-tile doors. WFC
  interior tiles will be introduced once the rectangular room graph and door
  system are stable.
- The live fog of war is in effect even in the prototype: no permanent
  explored map, only the current sensor picture.
- A basic hidden research system with diegetic research stations that unlock
  a few early tech-tree nodes to demonstrate progressive mechanics
  introduction.
- Ship templates and scenario data will be authored in Eldiron, demonstrating
  the separation between continuous ship composition and authored content.
- A minimal procedural generation algorithm registry with a default
  linear-room algorithm will be established to validate the first-class
  procgen concept.
- First-person drone camera and schematic overlay separation will be stubbed,
  with keyboard focus using `1`‑`9` and `0`, and function-key layer toggles.

This scope is intentionally small to prove the ECS, docking, physics, camera,
lighting, and sensor architecture before expanding content.

## 15. Actor Upgrades and Procedural Enemies

### Modular Actor Upgrades

Actors are not monolithic character classes. Each actor is a modular chassis
with a set of **upgrade modules**.

Upgrade examples:

- Camera upgrade
- Light upgrade
- Microphone upgrade
- Turret weapon upgrade
- Melee weapon upgrade
- Radar sensor upgrade
- Thermal sensor upgrade
- Movement legs upgrade
- Wheel drive upgrade
- Armor plate upgrade
- Battery upgrade
- Behavior upgrades (patrol, aggressive, flee, guard)

Upgrades can be mixed and matched. Player-controlled robots use the same
upgrade modules as enemies, but the player can see and choose them. For
enemies, upgrades are hidden and procedurally selected.

### Procedural Monster Generation

Enemies are procedurally generated actors, not bespoke monster types.

The enemy generator selects:

- A base chassis
- Sensory upgrades (camera, thermal, microphone, radar)
- Effector upgrades (weapons, tools, lights)
- Behavior upgrades
- Optional stat modifiers

This creates emergent enemy variety from a shared module pool. A “monster”
is simply the result of a generated set of upgrades.

### Behaviors as Upgrades

Behavior is also an upgrade. Instead of hardcoding enemy AI, each actor has
one or more behavior components supplied by behavior upgrades.

Example behavior upgrades:

- Patrol waypoints
- Guard radius
- Aggressive engage range
- Flee health threshold
- Scavenge target priorities

The combination of behavior upgrades defines the enemy’s personality and
tactics. A hidden set of behavior upgrades can produce surprising emergent
strategies.

### Player and Enemy Symmetry

Player robot units can also be upgraded with the same modules. This means
mechanics learned against enemies can be applied to player builds, and vice
versa. The shared module system keeps balance and emergent complexity under
control.

### Eldiron Role

Eldiron authors the upgrade templates, weighted generation tables, and
behavior transition rules. Bevy instantiates the upgrade modules into ECS
components at runtime.

## 15.5 Set Pieces as Agent Variations

Spacetrash treats many physical props, interactive objects, and environmental
set pieces as specialized agent variations rather than bespoke world objects.

This follows from the same ECS principle as upgrade-driven actors: an agent
is an entity assembled from a chassis plus optional upgrades, behaviors,
sensors, effectors, and physics components. A set piece is simply an agent
with some components omitted or replaced.

Examples:

- **Cargo box** is an entity with `PhysicsBody`, `CircleCollider` or
  `AabbCollider`, and `GameplayEntity`, but no chassis, no behavior, no
  sensor, no ray instrument, and no player input. It is a physical object,
  not a separate “prop” system.
- **Reactor core** is a room-center set piece with a collider and a basic
  `ReactorBehavior` or `ThermalSource` component. It does not move
  deliberately, but participates in physics and may have a simple behavior
  such as emitting heat, drawing power, or responding to damage.
- Door/airlock actuator nodes, fluid reservoirs, battery packs, and research
  stations are later represented the same way: physical objects with optional
  subsystem behavior, sharing collision and lifecycle code with actors.
- **Future bots** extend the same pattern by adding an internal battery/power
  storage and a chassis. The battery is itself an agent-like physical
  subsystem with capacity, thermal load, and fluid/energy ports. The bot uses
  the same physics-based mechanics as static set pieces, just with locomotion
  components.

Inspiration:
Factorio fluidboxes provide a useful model here:

- A fluidbox is a physical attachment point with capacity, connections, flow
  limits, and interactions.
- Fluidboxes are defined independently but attached to a machine/bot.
- The same fluidbox can be attached to a stationary tank, a moving train, or
  a portable bot.
- The machine/bot does not own the fluidbox code path; it only supplies
  connectivity and component placement.

Spacetrash should adopt an equivalent pattern:

- `PhysicalSetPiece` describes a base physical object: collider, mass, size,
  collision rules, and maybe socket/port count.
- Optional `AgentUpgradeSet` describes upgrades and behaviors that make the
  piece interactive or autonomous.
- Fluid/thermal/electrical ports become components shared by stationary set
  pieces, moving bots, and player drones.
- Behaviors are added per piece; a cargo box has none, a reactor has a simple
  thermal/electrical behavior, and a future bot can use the same port
  components while also having movement and behavior FSAs.

This keeps physics, systems, and interactions uniform. It also means editor
tools, map assertions, and headless arena tests can handle both actors and
props through the same entity lifecycle instead of maintaining separate prop
systems.

Implementation impact:

- `GameplayEntity` already provides common lifecycle.
- `PhysicsBody`, `CircleCollider`, `AabbCollider`, `ActorSize`, `Velocity`,
  `Rotation`, and `AngularVelocity` are reused.
- Optional `BehaviorDirective`, `AiBehaviorState`, and upgrade components are
  simply not inserted for inert set pieces.
- A new `PhysicalSetPiece` marker should be added to distinguish non-mobile
  interactive props from actual agents, but it must not bypass collision,
  lifecycle, or system graph integration.
- Future port components (`FluidPort`, `PowerPort`, `ThermalPort`) should be
  designed as ordinary components that can be attached to any entity,
  mirroring Factorio fluidboxes.
- Room-specific generation can guarantee a characteristic set piece by
  placing one at the deterministic center or another designated socket in
  the room, using the same graph constraints and terminal placement rules
  already defined.

## 16. Arcade Physics Formalization

For implementation details, see the mission plan section 12. The design
document formalizes the player experience:

- Movement feels responsive and slightly slidey.
- Actors have visible rotation; tank-style turning is expected.
- Collisions with walls are firm but not bouncy (low restitution).
- Knockback from damage is an immediate velocity change, not a simulated
  impulse.
- Upgrades change movement feel (wheel vs legs vs hover) by modulating
  friction, turn rate, and damping.
- Physics only affects movement and basic collision; it does not simulate
  torque, joint constraints, or complex material interactions.
