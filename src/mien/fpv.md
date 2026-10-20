# Focused View / First-Person Drone View Design

## Purpose

This document defines the **focused view** abstraction for drone/unit
control. It replaces the current hardcoded `FirstPersonCamera` assumption
with a modular, upgrade-driven view pipeline.

The focused view is the single continuous live feed available to the player.
It must support all future sensor and tool views, not only a visible-light
camera.

---

## Current State

The prototype currently has:

- `FirstPersonCamera` component on player/drone entities.
- `camera_follow_system` that moves a 3D world camera to the focused entity.
- `world_camera_visibility_system` that shows the camera only in
  `DiegeticView::FirstPerson`.
- hardcoded visible-light camera behavior.
- no active view selection.
- no LIDAR, thermal, radar, or blind view.

As a result, every drone effectively has the same camera, and upgrading a
drone does not change what the player sees.

---

## Design Goals

1. **Upgrade-driven**
   - What the player sees must be determined by the focused entity's
     installed upgrades.
   - A visible-light camera is only one possible focused view.

2. **Single live feed per player**
   - The player sees exactly one continuous feed at a time.
   - The active feed is selected by the focused drone's active view
     upgrade.

3. **Sensor-specific visual language**
   - Visible light, LIDAR, thermal, and radar must look and behave
     differently.
   - They must not all be fake variants of a regular 3D camera.

4. **Blind play is possible**
   - A drone without a ray instrument has no focused visual feed.
   - The player can still control movement and use terminal commands, but
     must rely on non-visual UI feedback.

5. **Deterministic and testable**
   - View selection is component/state driven.
   - The same world state and same active view produce the same feed.

6. **Terminal integration**
   - View switching must be available through the terminal:
     - `view camera`
     - `view lidar`
     - `view thermal`
     - `view radar`
     - `view none`

---

## Focused View Abstraction

The current `FirstPersonCamera` should be generalized into a view-upgrade
component.

Proposed components:

```rust
enum ViewUpgradeKind {
    None,
    VisibleLightCamera,
    Lidar,
    Thermal,
    Radar,
}

struct ActiveViewUpgrade {
    kind: ViewUpgradeKind,
}

struct ViewUpgrade {
    kind: ViewUpgradeKind,
    eye_offset: Vec3,
    fov: f32,
}
```

- `ViewUpgrade` describes what a drone has installed.
- `ActiveViewUpgrade` describes which installed view is currently selected.
- The focused view system queries the focused entity and selects the
  correct renderer based on `ActiveViewUpgrade`.

The legacy `FirstPersonCamera` component may remain temporarily for the
visible-light camera, but it should be refactored into this more general
model.

---

## View Modes

### 1. No Active View / Blind

**State:** `ViewUpgradeKind::None`

**Player experience:**

- The world camera is hidden.
- The focused screen shows no live visual feed.
- Terminal status and HUD are still visible.
- Movement commands still work.

This is the default state for a drone with no ray instrument.

**Visual:**

- Black or static screen.
- Strong indication that no active view is available.

---

### 2. Visible-Light Camera

**State:** `ViewUpgradeKind::VisibleLightCamera`

**Device model:**

- A ray instrument that detects visible light.
- Requires a light source in visible range.
- Dark rooms produce a black or heavily noisy image.

**Rendering model:**

- The existing 3D camera feed is used when sufficient visible light is
  present.
- Illumination comes from the shared sensor/light grid.
- If the local light value is below a threshold, the feed fades to black.

**Terminal command:**

```text
view camera
```

---

### 3. LIDAR

**State:** `ViewUpgradeKind::Lidar`

**Device model:**

- A geometric ray instrument.
- Does not depend on visible light.
- Returns coarse structural geometry, not identities or colors.

**Rendering model:**

- Top-down or first-person point-cloud/line view.
- Walls and obstacles are visible as bright lines or points.
- Objects without structural signature are not shown.
- Floors receive a base dark field.

**Terminal command:**

```text
view lidar
```

---

### 4. Thermal

**State:** `ViewUpgradeKind::Thermal`

**Device model:**

- A ray instrument measuring heat.
- Does not require visible light.
- Blocked or attenuated by insulated cells and some walls.

**Rendering model:**

- Heatmap overlay.
- Hot objects/rooms appear bright; cold areas appear dark.
- Powered systems, reactors, fires, and living enemies have strong signals.
- Static cold loot is weak or invisible.

**Terminal command:**

```text
view thermal
```

---

### 5. Radar

**State:** `ViewUpgradeKind::Radar`

**Device model:**

- A ray instrument that sends and receives radar pulses at intervals.
- Detects moving objects, but only weakly detects static structure.
- Returns stale pings between sweeps.

**Rendering model:**

- Radar scope view.
- Rotating sweep line and fading blips.
- Moving actors appear as blips; static structure is not a live image.
- Own drones may appear as steady markers.

**Terminal command:**

```text
view radar
```

---

## Active View Selection Rules

- Each drone has at most one active ray instrument.
- Selecting a view automatically deactivates the previous ray instrument.
- If the focused drone does not have the requested sensor upgrade, the
  terminal returns an error.
- If the drone has exactly one sensor upgrade, `view none` deactivates it.

Internally, the terminal writes the `ActiveViewUpgrade` component or an
`ActiveViewUpgradeChanged` event.

---

## Focused Entity + Active View System

The existing `FocusedEntity` resource remains the source of truth for which
drone the player is controlling.

The focused view system must:

1. Read `FocusedEntity`.
2. Read the focused entity's active view upgrade.
3. Run the appropriate renderer.
4. Do nothing if no active view is installed.

The current `camera_follow_system`, `world_camera_visibility_system`, and
`camera_follow_system` should be replaced by a small set of view-specific
systems:

```text
focused_view_selection_system
visible_camera_view_system
lidar_view_system
thermal_view_system
radar_view_system
blind_view_system
```

---

## Non-Ray Sensor Feedback

A drone without a ray instrument may still have non-ray sensors such as
microphones, motion detectors, or simple proximity sensors.

These sensors do not create a live visual feed, but they may create
terminal/HUD events:

```text
> motion detected northeast
> drone 2 pings hostile contact
```

The focused view design must therefore keep a clear boundary between:

- **live visual feed** — ray instrument view
- **sensor event output** — terminal/HUD text and schematic overlays

---

## Schematic Overlay Relationship

The terminal and schematic views are separate from the focused live feed.

- `/` opens terminal.
- Backtick opens schematic.
- Number keys focus drones.
- Focused sensor feed is only visible in first-person view.

The live sensor feed and the schematic map must share the same sensor grid
data, but they must remain visually distinct. The player cannot mistake a
LIDAR view for a structural schematic.

---

## HUD and Blind Indicators

The HUD must always show the current focused view:

```text
VIEW: VISIBLE LIGHT
CAMERA: NO SIGNAL
```

For no active view:

```text
VIEW: BLIND
CAMERA: NOT INSTALLED
```

For LIDAR:

```text
VIEW: LIDAR
GEOMETRY: LOCKED
```

For radar:

```text
VIEW: RADAR
SWEEP: 0.8s
LAST PING: 0.2s ago
```

These strings come from terminal-visible status, but the HUD may render
them as compact summaries.

---

## Data Flow

```text
UpgradeRegistry
    -> installs ViewUpgrade components at drone spawn
Terminal command "view thermal"
    -> writes ActiveViewUpgrade { kind: Thermal }
FocusedViewSystem
    -> reads FocusedEntity
    -> reads ActiveViewUpgrade
    -> chooses thermal renderer
ThermalRenderer
    -> samples shared sensor/thermal grid
    -> renders thermal heatmap
```

No view renderer should independently raycast the entire world each frame.
It must read from the shared sensor grid.

---

## Headless and Automated Testing

The focused view selection must be testable without rendering.

Tests should cover:

- spawning a drone with a visible-light camera and no light results in a
  dark/no-signal view.
- spawning a drone with LIDAR gives a geometry feed even in darkness.
- switching active view from camera to LIDAR changes the active view kind.
- attempting `view radar` without a radar upgrade fails cleanly.
- `view none` clears the active view.
- same drone state + same view command + same tick gives identical view
  metadata.

For headless testing, view rendering is not required. The test only needs
to verify the resulting `ActiveViewUpgrade` state and the sampled sensor
grid output.

Recommended test helper:

```rust
fn apply_terminal_command(app: &mut App, command: &str)
```

The focused-view metadata should be logged into the arena event log as part
of `ActorState` or a dedicated `FocusedViewChanged` event.

---

## Future View Upgrades

The focused view abstraction must support future upgrades without
rewriting the terminal or UI:

- `sonar` / acoustic view
- `magnetic anomaly view`
- `electrical field view`
- `radiation view`
- `x-ray backscatter`
- `remote camera relay`
- `spectral analyzer`

Each view has:

- a `ViewUpgradeKind`
- a renderer strategy
- a terminal command
- a HUD label
- a sensor-grid source
- a blind fallback when no signal

---

## Non-Goals

- The focused view is not a full-screen cinematic camera mode.
- It is not a replacement for the schematic map.
- It does not directly control movement.
- It does not grant perfect information; all data must come from the
  shared sensor grid and installed upgrades.
- It does not support multiple simultaneous visual feeds for one player.

The goal is to make every sensor/tool feel like an instrument, not to turn
the drone view into a conventional third-person game camera.
