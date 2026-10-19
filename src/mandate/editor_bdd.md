# In-Game Editor (God Mode) BDD

This document specifies the behavior of the in-game editor (god mode). The editor is not a separate `GameState`; it is a mode within `Playing` controlled by the `EditorMode` resource. While active, all gameplay simulation (input, movement, physics, AI, ship systems, combat) is paused. Only editor systems and the editor camera/UI run.

The editor is inspired by Factorio’s map editor: editing happens directly in the world, changes are immediate, and the simulation remains frozen until the player explicitly resumes.

## Feature: Entering and Exiting Editor Mode

### Scenario: Toggle editor mode with F1

- **Given** the game is in the `Playing` state
- **And** `EditorMode.active` is `false`
- **When** the player presses **F1**
- **Then** `EditorMode.active` becomes `true`
- **And** all gameplay systems are paused (they do not run while `EditorMode.active == true`)
- **And** the editor UI appears
- **And** the camera switches from follow-cam to editor free-cam

### Scenario: Toggle back to live play with F1

- **Given** `EditorMode.active` is `true`
- **When** the player presses **F1** again
- **Then** `EditorMode.active` becomes `false`
- **And** gameplay systems resume
- **And** the editor UI disappears
- **And** the camera returns to the normal follow-cam

## Feature: Editor Camera

### Scenario: Free camera movement

- **Given** editor mode is active
- **When** the player moves the mouse to the edges of the screen or uses WASD
- **Then** the camera pans smoothly across the world
- **And** the camera does not follow any particular entity

### Scenario: Camera zoom

- **Given** editor mode is active
- **When** the player scrolls the mouse wheel
- **Then** the camera zooms in or out
- **And** the zoom is clamped to reasonable minimum and maximum distances

### Scenario: Camera rotation (optional, later)

- **Given** editor mode is active
- **When** the player presses **Q** or **E**
- **Then** the camera rotates around the Y axis by 90 degrees
- **And** the world view updates accordingly

## Feature: Editor Tools and Interaction

### Scenario: Selecting an entity

- **Given** editor mode is active
- **And** the select tool is active
- **When** the player left-clicks on an entity (room, wall, door, docking port)
- **Then** that entity becomes highlighted
- **And** its properties are shown in the editor UI

### Scenario: Placing a new room

- **Given** editor mode is active
- **And** the room placement tool is active
- **When** the player drags a rectangle on the grid and releases
- **Then** a new room is created with the correct floor, walls, and collision
- **And** the room is added to a ship entity (either the current ship or a new one if none exist)
- **And** no simulation occurs

### Scenario: Placing a door

- **Given** editor mode is active
- **And** the door tool is active
- **When** the player clicks on a wall segment
- **Then** that wall segment is replaced by two shorter wall segments with a door gap of the configured door width
- **And** the door gap has no collider

### Scenario: Deleting an entity

- **Given** editor mode is active
- **And** the delete tool is active
- **When** the player left-clicks on an entity
- **Then** that entity and its children are despawned
- **And** any adjacent wall/door connectivity is updated to maintain a valid ship layout

### Scenario: Moving an entity

- **Given** editor mode is active
- **And** the move tool is active
- **When** the player selects an entity and drags it to a new position
- **Then** the entity’s transform is updated
- **And** if the entity is a room, its associated walls and floors move with it

## Feature: Editor UI

### Scenario: Editor UI appears on entering editor mode

- **Given** editor mode is activated
- **Then** a toolbar is displayed with buttons for each tool (select, room, door, delete, move)
- **And** a status bar shows the current tool and any relevant grid/snapping information
- **And** the UI does not block gameplay (since gameplay is paused)

### Scenario: Editor UI disappears on exiting editor mode

- **Given** editor mode is deactivated
- **Then** the editor UI is despawned
- **And** the normal HUD remains visible

## Feature: Saving and Loading While Editing

### Scenario: Saving the current world as a scenario

- **Given** editor mode is active
- **When** the player selects **Save Scenario**
- **Then** the current world is captured into a `Scenario` structure with:
  - all ship instances and their transforms
  - docking graph
  - system states
  - entity data
- **And** the scenario is serialized to disk
- **And** the editor remains active and the world is unchanged

### Scenario: Saving a ship template from the selected ship

- **Given** editor mode is active
- **And** a ship entity is selected
- **When** the player selects **Save Ship Template**
- **Then** the ship’s layout is saved as a `ShipTemplate` with the current rooms, doors, and docking ports
- **And** the template is added to the `ShipTemplateLibrary`

### Scenario: Loading a scenario while in editor

- **Given** editor mode is active
- **When** the player selects **Open Scenario** and chooses a file
- **Then** the current world is despawned
- **And** the loaded scenario is instantiated
- **And** editor mode remains active
- **And** the new world is paused

## Feature: Editor Constraints

### Scenario: No simulation while editing

- **Given** editor mode is active
- **When** time advances
- **Then** no gameplay systems (input, movement, physics, AI, ship system simulation, combat) execute
- **And** the world state does not change unless an editor tool explicitly modifies it

### Scenario: Editor cannot be activated from non-playing states

- **Given** the game is in `MainMenu`, `Paused`, or `GameOver`
- **When** the player presses **F1**
- **Then** editor mode does not activate
- **And** the state does not change

This document is a living specification. As editor features are implemented, update these scenarios to match actual behavior and to add more advanced features like blueprint import/export, multi-select, and undo/redo.
