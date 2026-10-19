# Main Game Loop and God Mode BDD

This document describes the expected behavior of the main gameplay loop, especially the interaction between normal play, pausing, and the in-game god-mode editor. It draws heavily from Factorio’s map editor experience: the world is paused while editing, changes are immediately reflected in the world, and the player can save/load without restarting.

## Feature: Entering God Mode During Play

### Scenario: Player enters the editor from the pause menu

- **Given** the game is in the `Playing` state
- **And** `EditorMode.active` is `false`
- **When** the player presses **Escape** to open the pause menu
- **And** the player selects **Enter Editor**
- **Then** `EditorMode.active` becomes `true`
- **And** the game returns to the `Playing` state
- **And** all gameplay systems (input, movement, camera follow, AI, ship system simulation, combat) stop running
- **And** the editor UI appears with tool palettes, selection outlines, and grid overlays
- **And** the camera switches from gameplay follow mode to an editor free-fly or top-down grid camera
- **And** the world remains exactly as it was at the moment of pausing

### Scenario: Player exits the editor back to live play

- **Given** `EditorMode.active` is `true`
- **And** the editor UI is visible
- **When** the player clicks the **Exit Editor** button
- **Then** `EditorMode.active` becomes `false`
- **And** gameplay systems resume from the current world state
- **And** the camera returns to the normal follow-cam mode
- **And** the editor UI is hidden

## Feature: Editing in God Mode

### Scenario: Player places a new room while in the editor

- **Given** god mode is active
- **And** the room placement tool is selected
- **When** the player drags a rectangle on the grid and releases
- **Then** a new room is spawned as an entity with the correct floor, walls, and door openings according to the current template/blueprint
- **And** the room’s `Room` component is attached
- **And** the room is assigned to a ship entity
- **And** no physics, AI, or system updates occur during the placement

### Scenario: Player deletes an existing room or wall

- **Given** god mode is active
- **And** the delete tool is selected
- **When** the player clicks on a room or wall entity
- **Then** that entity and its descendants are despawned
- **And** surrounding wall/door connectivity is updated if necessary
- **And** the world remains consistent for future gameplay

### Scenario: Player places a docking port

- **Given** god mode is active
- **And** the docking port tool is selected
- **When** the player clicks on a valid ship wall edge
- **Then** a `DockingPort` entity is created at that location with the correct `Facing`
- **And** the port is added to the ship’s port list
- **And** it becomes available for docking with other ships during play or future editing

### Scenario: Player edits ship systems in god mode

- **Given** god mode is active
- **And** the system editor panel is open
- **When** the player changes the atmosphere oxygen level, electrical charge, or fluid reservoir amount
- **Then** the relevant system component/resource is updated immediately
- **And** no simulation tick occurs while the game remains in editor mode

## Feature: Saving and Loading During Play/Editor

### Scenario: Player saves the current world as a new scenario

- **Given** the game is in either `Playing` or editor mode
- **When** the player selects **Save Scenario**
- **Then** the current world state is captured:
  - All ship instances (template id, world transform)
  - Docking graph
  - System states
  - Entity positions, inventories, and other runtime components necessary for resume
- **And** this data is serialized into a `Scenario` structure
- **And** the scenario is written to disk
- **And** the game continues in the same state (playing or editor) without interruption

### Scenario: Player loads a saved scenario during play

- **Given** the player is in the `Playing` state
- **When** the player selects **Load Scenario** from the pause menu and chooses a file
- **Then** the current world is despawned
- **And** the selected scenario is loaded:
  - If `world_state` is present, the world is restored directly from snapshot data
  - Otherwise, the scenario’s ship templates and docking links are used to regenerate the world
- **And** the game remains in `Playing` state with simulation running

### Scenario: Player saves the current ship as a reusable template (blueprint)

- **Given** the player is in editor mode
- **And** a ship entity is selected
- **When** the player chooses **Save Ship Template**
- **Then** the selected ship’s layout is captured as a `ShipTemplate`:
  - Procedural seed and generation parameters are retained if the layout was generated
  - Manual rooms, door overrides, and docking ports are stored as explicit overrides
- **And** the new template is added to the `ShipTemplateLibrary`
- **And** the template can be used later when starting a new scenario or spawning a new ship

## Feature: Starting a New Scenario from Within the Editor

### Scenario: Player clears the world and starts a blank scenario in editor

- **Given** god mode is active
- **When** the player selects **New Scenario**
- **Then** all current `GameplayEntity` entities are despawned
- **And** a new empty `Scenario` is loaded
- **And** `EditorMode` remains `true`
- **And** the player can begin building from scratch

### Scenario: Player replaces the current world with a different scenario in editor

- **Given** god mode is active
- **When** the player selects **Open Scenario in Editor** and chooses a file
- **Then** the current world is despawned
- **And** the chosen scenario is loaded with `EditorMode` kept `true`
- **And** all simulation remains paused

## Feature: Returning to Main Menu from Gameplay or Editor

### Scenario: Player exits to main menu while in normal play

- **Given** the game is in `Playing`
- **When** the player presses **Escape** to open the pause menu and selects **Main Menu**
- **Then** the world is despawned using `despawn_gameplay_world`
- **And** `EditorMode` is reset to `false`
- **And** the game transitions to `MainMenu`

### Scenario: Player exits to main menu while in editor mode

- **Given** `EditorMode.active` is `true`
- **When** the player selects **Exit to Main Menu** from the editor UI
- **Then** the world is despawned
- **And** `EditorMode` is reset to `false`
- **And** the game transitions to `MainMenu`

## Feature: Pause Menu Behavior in Normal Play

### Scenario: Pausing and resuming from pause menu

- **Given** the game is in `Playing` and editor mode is off
- **When** the player presses **Escape**
- **Then** the game transitions to `Paused`
- **And** the pause menu is shown
- **And** gameplay systems are paused by the state change

- **When** the player selects **Resume** or presses **Escape** again
- **Then** the game transitions back to `Playing`
- **And** gameplay resumes

### Scenario: Entering editor from pause menu

- **Given** the game is in `Paused`
- **When** the player selects **Enter Editor**
- **Then** the game transitions to `Playing` with `EditorMode` set to `true`
- **And** the pause menu is closed
- **And** the editor UI appears
- **And** the world remains paused in editor mode

## Feature: Diegetic Command Interface

### Scenario: Focusing on a drone using number keys

- **Given** the game is in `Playing` and the command interface is active
- **When** the player presses one of the number keys `1` through `9`
- **Then** the view centers on the corresponding drone unit
- **And** that drone becomes the currently controlled unit for movement and commands
- **And** the HUD updates to show the selected drone’s status and inventory
- **And** if the commander is currently in the command line, the pressed number may also be used as command input; the focus change still occurs and does not interfere with command parsing

### Scenario: Focusing on the Blackbox using 0

- **Given** the game is in `Playing`
- **When** the player presses number key `0`
- **Then** the view centers on the player’s main character, the **Blackbox**
- **And** the Blackbox becomes the currently controlled unit
- **And** the HUD updates accordingly

### Scenario: Toggling between command line and schematic map

- **Given** the game is in `Playing`
- **When** the player presses the backtick (`` ` ``) key
- **Then** if the command line was active, it closes and the schematic map opens
- **And** if the schematic map was active, it closes and the command line reopens
- **And** the active view updates immediately; the world simulation continues normally

### Scenario: Switching schematic layers with function keys

- **Given** the schematic map is open
- **When** the player presses a function key from `F1` through `F12`
- **Then** the schematic map switches to the corresponding system layer:
  - `F1` structural
  - `F2` electrical
  - `F3` fluidics
  - `F4` atmospherics
  - `F5` thermal
  - `F6` computer/network
  - `F7` research/tech
  - `F8` docking/travel
  - `F9` sensors/tactical
  - `F10`‑`F12` reserved for future layers
- **And** the schematic map visual representation updates to show only the selected layer’s information, with other layers hidden or dimmed

### Scenario: Closing the schematic map

- **Given** the schematic map is open
- **When** the player presses the backtick key again or presses `Escape`
- **Then** the schematic map closes and the command line returns
- **And** the world view returns to the normal live camera

## Notes

- God mode is not a separate game state; it is a mode within `Playing` controlled by `EditorMode`. This allows the world to stay instantiated without fighting state-transition despawn logic.
- All gameplay systems must respect the run condition `playing_and_not_editor` so they stop when editor mode is active.
- The editor UI and camera should be implemented later but the BDD scenarios describe the intended behavior now.
- Factorio’s ability to edit an existing save without loading into a separate “editor executable” is the target: entering the editor pauses the sim and turns on building tools, then exiting resumes.
- The diegetic command interface uses fixed reserved keys: number keys `1`‑`9` and `0` for focus, backtick for command/schematic toggle, and function keys `F1`‑`F12` for schematic layers. These bindings are part of the core gameplay contract and must not be remapped by mods or future features.
- Editor mode is toggled via the pause menu button **Enter Editor** and exited via the editor UI button **Exit Editor**. No keyboard shortcut is currently assigned.
