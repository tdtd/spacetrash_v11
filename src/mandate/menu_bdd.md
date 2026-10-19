# Menu Behavior Driven Development

This document describes the expected behavior of the main menu and related loading/starting flows using BDD-style scenarios.

Factorio provides the inspiration: the menu is a lightweight entry point that can launch either a live play session or the editor/god-mode environment. The same scenario files can be opened in either mode.

## Feature: Starting a New Game

### Scenario: Player starts a new game with the default scenario

- **Given** the game is on the main menu
- **And** no scenario is currently loaded in memory
- **When** the player selects **Start Game**
- **Then** the game transitions to the `Playing` state
- **And** the default scenario is loaded
- **And** all ships defined by that scenario are instantiated at their configured world positions
- **And** any initial docking links are established
- **And** the player-controlled character is spawned at the scenario’s default spawn point
- **And** the in-game HUD is visible

### Scenario: Player starts a new game while god mode was previously active

- **Given** the game is on the main menu
- **And** the `EditorMode` resource is currently `true` from a previous editor session
- **When** the player selects **Start Game**
- **Then** `EditorMode` is set to `false`
- **And** the game transitions to `Playing` with simulation running normally

## Feature: Opening the Ship/Scenario Editor from the Main Menu

### Scenario: Player opens the editor directly from the main menu

- **Given** the game is on the main menu
- **When** the player selects **Open Editor**
- **Then** the game transitions to the `Playing` state
- **And** `EditorMode` is set to `true`
- **And** the default or last-used scenario is loaded
- **And** all gameplay simulation systems (input, movement, AI, ship systems) are paused
- **And** the editor UI becomes visible
- **And** the camera switches to an editor-style camera (free-fly or top-down grid view)
- **And** the player can begin placing, moving, and deleting ship/room/door entities

### Scenario: Player opens the editor with a blank scenario

- **Given** the game is on the main menu
- **And** the player wants to start from an empty world
- **When** the player selects **New Scenario in Editor**
- **Then** the game transitions to `Playing` with `EditorMode` enabled
- **And** the scenario is initialized with no ships and no docking links
- **And** the world is cleared of all `GameplayEntity` entities
- **And** the editor UI is shown

## Feature: Loading an Existing Scenario

### Scenario: Player loads a scenario into the live game

- **Given** the game is on the main menu
- **And** a valid scenario file exists on disk
- **When** the player selects **Load Scenario** and chooses the file
- **Then** the game transitions to the `Playing` state
- **And** `EditorMode` is set to `false`
- **And** the scenario’s ships are spawned at their saved positions
- **And** docking links are restored
- **And** if `world_state` is present, it is used to restore entity/system states instead of regenerating from templates
- **And** the game resumes with simulation running

### Scenario: Player loads a scenario into the editor

- **Given** the game is on the main menu
- **And** a valid scenario file exists
- **When** the player selects the scenario and chooses **Open in Editor**
- **Then** the game transitions to `Playing` with `EditorMode` enabled
- **And** the scenario’s world is loaded, but all simulation systems remain paused
- **And** the editor UI is shown

## Feature: Returning to the Main Menu

### Scenario: Player quits to main menu from the editor or game

- **Given** the game is in `Playing`, `Paused`, or editor mode
- **When** the player selects **Main Menu** from the pause menu or editor UI
- **Then** all `GameplayEntity` entities are despawned
- **And** `EditorMode` is reset to `false`
- **And** the game transitions to the `MainMenu` state
- **And** the main menu UI is visible

## Feature: Quitting the Game

### Scenario: Player exits the application from the main menu

- **Given** the game is on the main menu
- **When** the player selects **Quit**
- **Then** the application exits cleanly

### Scenario: Player quits while in game or editor

- **Given** the game is in `Playing`, `Paused`, or editor mode
- **When** the player selects **Quit to Desktop** from the pause menu or editor UI
- **Then** the application exits cleanly

## Feature: Menu Navigation and State Transitions

### Scenario: Main menu button interactions change game state

- **Given** a visible main menu
- **When** the player presses **Enter** or **Space**
- **Then** the game starts a new game using the default scenario

- **When** the player presses **Escape**
- **Then** the game exits

- **When** the player clicks a button that is assigned to start/load/open editor
- **Then** the corresponding state transition occurs as described in the scenarios above

## Notes

- The menu never directly spawns gameplay entities. Loading is handled by `OnEnter(Playing)` via `setup_scene` or the editor loading path.
- Scenario files can be opened as either playable live worlds or paused editor worlds. The distinction is controlled by `EditorMode` at load time.
- All menu-to-playing transitions must clear any previous world state unless the scenario explicitly restores it.
- Factorio-inspired blueprint/blueprint-import functionality will be added to the menu later, but menu behavior must support launching the editor from the main menu from the start.
