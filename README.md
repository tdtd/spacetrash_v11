# spacetrash_v11

**spacetrash_v11** is a continuous, real-time space salvage adventure built in Rust with the Bevy engine. The player commands a small salvage ship, a main character called the **Blackbox**, and up to nine drones. Exploration takes place across procedurally generated spaceships that can dock together into one evolving playfield.

Eldiron is used as the authoring and scenario-creation layer, while Bevy owns the runtime simulation. Ship templates and scenarios are authored separately, allowing ships to be procedurally generated, manually edited, and reused across many different scenarios.

## Current Prototype

The current prototype includes:

- Bevy app with game states: `Loading`, `MainMenu`, `Playing`, `Paused`, `GameOver`, `Victory`
- Entity-component structure for ships, rooms, docking ports, drones, and the Blackbox
- Lightweight arcade physics with velocity, damping, and simple AABB collision resolution
- A basic ship template library and ship generator
- Main menu, pause menu, loading screen, game-over screen, and in-game HUD
- God-mode editor activated from the pause menu
- Diegetic command/schematic interface:
  - `1`–`9` focus individual drones
  - `0` focuses the Blackbox
  - Backtick toggles between command line and schematic map
  - `F1`–`F12` switch schematic layers
- Fallback schematic bindings using `Ctrl+1` through `Ctrl+9`
- First-person camera component and world-camera feed separation planned and partially implemented

## Controls

| Key | Action |
| --- | --- |
| `WASD` / Arrow keys | Move the currently focused unit |
| `1`–`9` | Focus a drone |
| `0` | Focus the Blackbox |
| `` ` `` | Toggle command line / schematic map |
| `F1`–`F12` | Switch schematic layer |
| `Ctrl+1`–`Ctrl+9` | Fallback schematic layer selection |
| `Esc` | Pause menu |
| `K` | Debug game-over screen |

Editor mode is currently entered from the pause menu using **Enter Editor**, and exited using the editor UI **Exit Editor** button.

## Getting Started

### Prerequisites

- Rust and Cargo installed via [rustup](https://rustup.rs)

### Run the Prototype

```bash
cargo run
```

The game will open a Bevy window and start in the main menu.

## Project Structure

```text
spacetrash_v11/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs
    ├── app.rs
    ├── components/
    │   └── mod.rs
    ├── resources/
    │   ├── mod.rs
    │   ├── ship_template.rs
    │   ├── scenario.rs
    │   └── procgen.rs
    ├── systems/
    │   ├── mod.rs
    │   ├── setup_scene.rs
    │   └── ship_generation.rs
    ├── ui/
    │   ├── mod.rs
    │   ├── loading.rs
    │   ├── main_menu.rs
    │   ├── pause.rs
    │   ├── game_over.rs
    │   ├── hud.rs
    │   └── editor.rs
    ├── mien/
    │   └── design.md
    ├── meister/
    │   └── architecture.md
    ├── mission/
    │   └── index.md
    └── mandate/
        ├── menu_bdd.md
        ├── main_game_loop_bdd.md
        └── editor_bdd.md
```

## Documentation

- **Design:** `src/mien/design.md`
- **Architecture:** `src/meister/architecture.md`
- **Mission plan:** `src/mission/index.md`
- **Behavior-driven specifications:** `src/mandate/`

## Next Steps

The current prototype focuses on:

1. Ship template and scenario management
2. Procedural generation algorithms as first-class objects
3. God-mode editor workflows
4. First-person drone camera and schematic overlay separation
5. Live fog of war and sensor-backed visibility

See `src/mission/index.md` for the full roadmap.

## License

This project is currently unlicensed. Contact the maintainer for details.
