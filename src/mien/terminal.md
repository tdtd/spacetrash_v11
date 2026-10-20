# Terminal Command Interface Design

## Purpose

The command terminal is the primary diegetic control surface for the drone
fleet. It is inspired by **Duskers**, but it is not a shell emulator used for
flavor. It is the real control path for issuing orders, switching active
tools, querying drone state, and managing behavior.

The terminal must remain lightweight enough to drive the first-person fleet
experience without requiring a mouse. It must also be deterministic and
testable, because the same command parser will be used in headless arena
episodes and automated search.

---

## Current State

The prototype currently has:

- `DiegeticView::CommandLine`
- a simple `CommandLineScreen` UI
- `update_command_line_focus_text`
- hardcoded first-person camera focus through `FirstPersonCamera`
- no command parser
- no terminal command history
- no terminal command execution

The terminal is therefore currently a placeholder screen, not a usable
control surface.

---

## Design Goals

1. **Diegetic, not magical**
   - Commands should read like drone-control instructions.
   - They should map to concrete components and systems.
   - They should not expose raw ECS internals to the player.

2. **Keyboard-first**
   - The terminal is opened with `/`.
   - Backtick toggles between terminal and schematic.
   - Number keys remain reserved for direct focus switching.
   - The terminal must be usable without clicking.

3. **Terminal input capture**
   - Pressing `/` focuses the command terminal.
   - Once the terminal input buffer is non-empty, the terminal captures all
     keys, including number keys.
   - Number keys `1`-`9` and `0` switch drone/Blackbox focus only when the
     terminal input buffer is empty.
   - Because of this, a terminal command cannot begin with a numeric digit.
     Leading numeric input is parsed as an unknown command.

4. **Deterministic**
   - The same text command in the same world state must produce the same
     effect.
   - Command parsing must be unit-testable without running the full game.

5. **Upgrade-aware**
   - The terminal must not assume every drone has a visible-light camera.
   - Commands must check the focused drone's installed upgrades.
   - If a command requires a missing upgrade, the terminal must return a
     meaningful error.

6. **Extensible**
   - New upgrade modules may introduce new terminal commands.
   - New sensor/tool view modes must have command equivalents.
   - The parser should be data-driven enough to avoid hardcoding every new
     tool in unrelated UI systems.

---

## Interface Layout

The existing `CommandLineScreen` should remain the visual foundation, but
it must evolve into a small terminal emulator:

```text
COMMAND LINE

Focused: Drone 1

> help

Available commands:
  focus <1-9 | blackbox>
  status
  scan
  view <camera | lidar | thermal | radar | none>
  active <upgrade-id>
  deactivate <upgrade-id>

  move <x> <z>
  patrol <waypoint> ...
  guard <x> <z> [radius]
  follow <entity>
  scavenge
  aggressive
  flee
  passive

  open <airlock>
  close <airlock>
  dock <port>
  pump <source> <destination>
  hack <target>

> view thermal
DRONE-1: thermal view active
```

The most recent command prompt remains at the bottom, while older output
scrolls upward.

---

## Command Categories

### 1. Focus and Selection

These commands change the active drone or query the currently controlled
unit.

| Command | Aliases | Effect |
| --- | --- | --- |
| `focus 1` | `f 1` | Focus Drone 1 and switch to first-person/focused view. |
| `focus blackbox` | `fb` | Focus the Blackbox. |
| `focus 0` | `f 0` | Focus the Blackbox as a numeric alias. |

Direct number keys `1` through `9` and `0` remain the fastest focus
shortcut, but the terminal must support the same action.

### 2. Status and Diagnostics

| Command | Aliases | Effect |
| --- | --- | --- |
| `status` | `st` | Show current chassis, upgrades, behavior directive, power, inventory, and active view. |
| `status all` | `st all` | Show a compact status table for all drones. |
| `ping` | `p` | Emit a sensor ping if the drone has a compatible ray or non-ray sensor. |

`status all` is important for fleet management and should be the first
non-trivial command implemented.

### 3. Active View and Sensor Control

The terminal is the primary way to swap a drone's active ray instrument.

| Command | Effect |
| --- | --- |
| `view camera` | Activate the visible-light camera feed. Fails if no visible-light receiver is installed. |
| `view lidar` | Activate LIDAR view. Fails if no LIDAR upgrade is installed. |
| `view thermal` | Activate thermal view. Fails if no thermal sensor is installed. |
| `view radar` | Activate radar view. Fails if no radar sensor is installed. |
| `view none` | Deactivate all active ray instruments. The player pilots blind but may still issue movement orders. |

A drone may have only one active ray instrument. Activating a new ray
view automatically deactivates the previous ray instrument.

### 4. Behavior and Autonomy Directives

These commands set `BehaviorDirective` on the focused drone.

| Command | Effect |
| --- | --- |
| `patrol <x> <z> [x2 z2 ...]` | Set `BehaviorDirective::Patrol` and supply waypoints. |
| `guard <x> <z> [radius]` | Set `BehaviorDirective::Guard` at the given location. |
| `follow <1-9>` | Set `BehaviorDirective::FollowLeader` and target another drone or Blackbox. |
| `scavenge` | Set `BehaviorDirective::Scavenge`. |
| `aggressive` | Set `BehaviorDirective::Aggressive`. |
| `flee` | Set `BehaviorDirective::Flee`. |
| `passive` | Set `BehaviorDirective::Passive` and clear autonomous behavior. |

These commands do not install permanent upgrades. They issue directives.
If the drone lacks the corresponding behavior upgrade, the command must
fail with a clear message:

```text
> guard 4 4
DRONE-1: guard behavior is not installed
```

The terminal is therefore the player-facing interface for the existing
`BehaviorDirective` component and the future upgrade-aware arbitration.

### 5. Movement Orders

Movement orders are useful for issuing short point-to-point commands without
manually piloting every tick.

| Command | Effect |
| --- | --- |
| `move <x> <z>` | Set a direct movement goal for the focused drone. |
| `goto <room-name>` | Navigate through the room graph toward a named room. |
| `stop` | Clear the current movement goal. |

The prototype may start with `move`. `goto` depends on room-graph
pathfinding and should be introduced after the typed room graph is stable.

### 6. Actuator and Tool Commands

Tool commands activate/deactivate equipment that is not a ray instrument.

| Command | Effect |
| --- | --- |
| `active <upgrade-id>` | Activate a swappable upgrade such as a light, grabber, or repair tool. |
| `deactivate <upgrade-id>` | Deactivate the upgrade. |
| `use <upgrade-id> [target]` | Use the named tool/effector. |

Examples:

```text
active cargo_bay
use repair_tool room3
deactivate cargo_bay
```

### 7. Docking and Access Commands

The terminal should provide deterministic control over physical ship
features.

| Command | Effect |
| --- | --- |
| `open <airlock-id>` | Open a docking airlock. |
| `close <airlock-id>` | Close a docking airlock. |
| `dock <port-id>` | Request docking with a compatible port. |
| `undock <port-id>` | Disconnect from a docked port. |
| `pump <source> <destination>` | Transfer fluid if valid, secured path exists. |
| `hack <target>` | Start a computer/network action. |

These commands depend on future ship-system components and are listed here
so the parser grammar is not redesigned later.

### 8. Command Line History and Help

The terminal must maintain a small command history.

Required behavior:

- `up` / `down` navigate previous commands.
- `clear` clears the visible output.
- `help` lists available commands.
- `help <command>` explains one command.

History is local to the current play session and is not part of saved world
state.

---

## Parser Design

The parser should be simple and deterministic.

Input is represented as:

```rust
enum TerminalCommand {
    Focus(Target),
    Status(Option<StatusTarget>),
    Ping,
    SetView(ViewMode),
    SetBehavior(BehaviorDirective),
    Move { x: f32, z: f32 },
    Goto(String),
    Stop,
    Activate(String),
    Deactivate(String),
    Use { upgrade: String, target: Option<String> },
    Open(String),
    Close(String),
    Dock(String),
    Undock(String),
    Pump { source: String, destination: String },
    Hack(String),
    Help(Option<String>),
    Clear,
}
```

Parsing rules:

- Commands are case-insensitive.
- Tokens are split on whitespace.
- Quoted identifiers are supported for room names and upgrade IDs.
- Unknown commands produce an error but never panic.
- Parsing is a pure function so it can be unit-tested.
- A command cannot begin with a numeric digit.

Example pure parser:

```rust
fn parse_terminal_command(input: &str) -> Result<TerminalCommand, CommandError>
```

This function must not read ECS state. Execution happens in a separate
system.

---

## Execution Model

Command execution is split into phases:

1. Parse the input string.
2. Resolve the focused entity and relevant upgrade availability.
3. Validate command against installed components.
4. Mutate components/resources via a command event or system.
5. Append a result message to the terminal output.

A command should never directly touch transform data unless it is a movement
order. It should instead write a component or resource consumed by the
appropriate gameplay system.

Recommended event:

```rust
struct TerminalCommandEvent {
    source_entity: Entity,
    command: TerminalCommand,
    tick: u64,
}
```

The event log can record this for arena testing and behavior descriptors.

---

## Upgrade Command Registration

New upgrade definitions should be able to add command names without editing
the core terminal internals.

Suggested extension point:

```rust
struct TerminalCommandSpec {
    name: &'static str,
    aliases: &'static [&'static str],
    required_components: Vec<ComponentId>,
    parser: ParsedCommand,
}
```

The upgrade registry can later provide these specs. For the first version,
a hardcoded command table is acceptable if it remains isolated in one module.

---

## Multi-Drone Fleet Behavior

The terminal is not only for the focused drone. Fleet-wide commands are
needed for Duskers-like control.

Planned syntax:

```text
all status
all scavenge
all follow blackbox
1..3 guard 4 4
```

The parser should support:

- `all <command>` — apply to all controllable drones.
- `<range> <command>` — apply to a drone range.
- `1,3 <command>` — apply to a comma-separated list.

This is a later grammar extension. The initial parser should reserve the
syntax but only implement `focus`, `status`, and single-drone behavior
commands.

---

## Terminal State and Resources

The terminal should be backed by resources, not private UI state.

Suggested resources:

```rust
struct TerminalInputBuffer {
    current: String,
    history: Vec<String>,
    cursor: usize,
}

struct TerminalOutputLog {
    lines: VecDeque<String>,
}
```

This keeps the terminal deterministic and testable.

The existing `CommandLineScreen` should read these resources and render them
without simulating the actual command logic.

---

## Headless Testing

Terminal parsing and execution must be testable without rendering.

Suggested tests:

- `parse_help_command`
- `parse_focus_drone`
- `parse_behavior_directive`
- `unknown_command_is_rejected`
- `missing_upgrade_command_fails`
- `behavior_directive_applied_to_correct_drone`
- `command_history_round_trip`
- `command_cannot_begin_with_number`

The headless arena runner can execute a scripted command list as the player
input interface. This is essential for automated behavior search.

---

## Visual Style

The terminal should feel like an old drone-control terminal:

- dark background
- monospaced font
- green or amber phosphor text
- visible prompt `> `
- subtle terminal line-spacing
- no graphical mouse cursor in the terminal view

The same font/style should be reused by the schematic UI where appropriate,
but the terminal remains a text-first interaction mode.

---

## Future Work

- command aliases and command macros
- saved command profiles
- mission/order queueing
- remote command routing through the computer network
- terminal plugins supplied by upgrade definitions
- full `goto`, `hack`, `pump`, and `dock` execution

These features build on the same parser/execution boundary and must not
introduce direct UI-to-world mutations.

---

## Non-Goals

- The terminal is not a general-purpose shell or programming language.
- It is not a replacement for the schematic map.
- It is not intended to display raw Bevy component names.
- It does not directly perform raycasts or sensor propagation; it only
  writes gameplay-relevant commands/components.

The terminal is the player-facing command surface for the upgrade-driven
drone fleet. Every future tool, sensor, and behavior should be accessible
through this interface.
