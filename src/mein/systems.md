# Schematic Layer Visual Design — Ambitious Multi-Mode Spec

This document replaces the earlier ASCII-only layer visual design with an
ambitious multi-mode system. The structural layer remains an ASCII roguelike
map, but every other layer is assigned a distinct **rendering mode** that
goes far beyond swapping glyphs and colors. The goal is to make each layer
feel like a completely different instrument panel, not just a recolored map.

The current prototype renders the schematic map as a single monospaced
`Text` string with one foreground color and one background color. This is
sufficient for the structural layer, but it cannot support vector lines,
pixel-art flow, thermal gradients, or radar sweeps. This document therefore
defines both the desired visual identity for each layer and the technical
extensions needed to realize it.

---

## 1. Current Drawing Primitives (Prototype Reality)

- **One string per whole map** — produced by `update_ascii_schematic_map`.
- **Single foreground color** — set via `TextStyle.color` on the
  `SchematicAsciiText` entity.
- **Single background color** — currently a `NodeBundle` background on the
  containing UI node, not per cell.
- **No fonts other than the default Bevy font** — we do not yet load a true
  monospaced font, but we rely on monospace-like alignment.
- **No images, vector meshes, per-cell colors, shaders, or custom UI nodes**
  — all layers currently share the same ASCII grid renderer.

These constraints are too limited for the ambitious layer identities below.
We therefore define a **rendering mode** per layer and a phased extension
path.

---

## 2. Design Principles

1. **Structural stays ASCII** — `#`, `.`, `+`, drones as digits. It is the
   reference map and will not change.
2. **Every other layer uses a distinct visual metaphor** — not just a glyph
   swap.
3. **No ASCII fallback for non-structural layers.** The structural layer is
   the only ASCII layer. All other layers must be visually distinct from
   structural and from each other through their own iconography, styling,
   and rendering mode.
4. **Color is emotional and status-driven, not merely decorative.
5. **Animation is allowed, but deterministic and often slow** — for example,
   a slow electrical pulse or fluid ripple.
6. **Layer-specific rendering modes are isolated** — adding a mode for one
   layer must not affect any other layer.
7. **Accessibility is handled through distinct visual languages and
   colorblind-safe patterns**, not by degrading a layer to ASCII.

---

## 3. Proposed Layer Rendering Modes

Below is the core mapping of layers to rendering styles.

| Layer        | Rendering Mode                                  | Analogy                                 |
|--------------|-------------------------------------------------|-----------------------------------------|
| Structural   | ASCII terminal map                               | Roguelike minimap                       |
| Electrical   | Vector circuit schematic                         | PCB / wiring diagram                    |
| Fluidics     | Pixel-fluid animation                            | Sand/water simulation                    |
| Atmospherics | Isopleth pressure map                            | Weather contour map                     |
| Thermal      | Thermal heatmap with animated gradients          | Predator vision / infrared camera        |
| Computer     | Command-line terminal / network explorer         | Shell output / traceroute map           |
| Research     | Constellation diagram with node pulses           | Star map / skill tree                    |
| Docking      | Blueprint drawing with dashed lines and arrows   | Engineering drawing / Factorio blueprint |
| Sensors      | Radar sweep overlay with polar ping visuals      | Submarine sonar / tactical scope         |
| Reserved     | Static / unknown fog                                 | Scan static / encrypted placeholder      |

Each mode is described in detail below.

---

## 4. Layer Detailed Visual Language

### 4.1 Structural — ASCII Roguelike (Unchanged)

**Mode:** Classic ASCII terminal map.

- `#` — wall
- `.` — floor
- `+` — door
- `@` — Blackbox
- `1`–`9` — drones

**Font:** True monospaced font (must be loaded in Phase 2).
**Color:** Pale gray with optional dark background.

This layer remains the simplest and most readable. It is the only ASCII
layer and the baseline for all other views.

### 4.2 Electrical — Vector Circuit Schematic

**Visual metaphor:** Thin glowing traces, small component nodes, energised
pulse animation.

**How it should look:**
- Background is very dark navy.
- Conductive paths are drawn as **continuous colored lines** (not `=` glyphs) 
  between room centers. They may be straight or have 45° bends.
- Lines have a subtle neon glow.
- Terminals are small **circle nodes** at room centres.
- Reactors are marked with a **pulsing bright circle** and `R` glyph inside
  **only as a small annotation, not as the primary representation**.
- A slow energy pulse (bright spot) travels along powered paths every few
  seconds.
- Broken/unpowered paths are dashed gray lines instead of solid cyan.

**Implementation extension required:**
- Need a custom vector overlay: either a `bevy_prototype_lyon` style 2D line
  renderer, or a procedural mesh/`ExtrudedCurve`, or a shader that draws
  lines into the UI.
- No ASCII glyphs should be used for the electrical layer. The `R`, `G`, `P`
  annotations in the old ASCII style are not part of this mode.

### 4.3 Fluidics — Pixel-Fluid Simulation

**Visual metaphor:** Blocky animated liquid, like a low-res water simulation.

**How it should look:**
- Room interiors are represented by a coarse **pixel grid** (e.g., 4x4 per
  tile) instead of single characters.
- Pipes appear as connected **blocky channels** of animated blue/green pixels.
- Reservoir/tank is a larger filled block.
- Pumps are rotating pixel icons (`P` with alternating background) **but only
  as pixel-art symbols**, not as ASCII text.
- Valve states are shown by colored squares.
- Flow direction is indicated by moving particles or a wave pattern along
  pipes.

**Implementation extension required:**
- This is a completely different renderer from ASCII. We could use a grid of
  colored `UI Nodes` per cell, or a small CPU-based pixel buffer drawn to a
  texture.
- No ASCII fallback should exist; the pixel-fluid mode is the canonical
  visual representation for this layer.

### 4.4 Atmospherics — Isopleth Pressure Map

**Visual metaphor:** Weather map with pressure contour lines and air flow
arrows.

**How it should look:**
- The map is divided into room-sized cells, each filled with a **colored band**
  representing pressure (cool to warm).
- Thin **contour lines** separate bins.
- Vents are marked with small **fans** or arrows showing flow direction.
- Oxygen-rich areas glow softly; CO₂/contaminant areas have a sickly tint.
- Animated streamlines follow pressure gradients between open doors.

**Implementation extension required:**
- Needs per-cell background colors. Many UI `NodeBundle` children can be
  used as colored tiles, or a custom shader.
- Arrows/streamlines could be vector overlays, not ASCII `^` or `+`.

### 4.5 Thermal — Animated Heatmap

**Visual metaphor:** Infrared camera view with temperature gradients.

**How it should look:**
- Background is a **heat gradient** (black-blue-red-yellow-white).
- Rooms with high heat appear as bright hotspots.
- Reactors are intense white cores.
- Radiators look like cool striped objects.
- A slow animated heat bloom expands around hot sources.
- No glyphs (`H`, `~`, etc.) are used; the heatmap itself communicates
  intensity.

**Implementation extension required:**
- A real heatmap shader/colored gradient. Could start with per-cell background
  colors using a continuous color ramp.
- Animated by low-frequency modulation.
- No ASCII `░▒▓` fallback.

### 4.6 Computer — Command-Line Terminal

**Visual metaphor:** Text-based network explorer / traceroute output within
the schematic panel.

**How it should look:**
- Instead of a map drawing, the screen shows a **command-line transcript**.
- It lists discovered network nodes, connections, and system states as text.
- Example:
  ```
  $ netmap --live
  > Bridge............192.168.1.1   [UP]
  > ComputerCore......192.168.1.4   [UP]
  > Storage...........192.168.1.9   [DOWN]
  $ trace route from Bridge to Reactor
  Bridge -> Engineering -> Reactor   [OK]
  ```
- This mode **is itself terminal/ASCII in aesthetic**, but it is not a
  fallback of another layer; it is the canonical computer/network view.
- Color is monospace green or amber phosphor on black.
- A blinking cursor at the bottom.

**Implementation extension required:**
- No geometric map needed. This can reuse the existing `Text` component but
  with a different generated string and maybe a separate UI entity.
- The game already has a command line overlay; this layer can piggyback on
  that aesthetic.

### 4.7 Research — Constellation Diagram

**Visual metaphor:** Star map like a tech tree/skill constellation.

**How it should look:**
- Room positions are used as anchors, but instead of a map they are drawn as
  **small stars/points**.
- Connections are thin dotted lines.
- Each research node is a symbol:
  - `♦` for a discovered tech node
  - `?` for unknown
  - `!` for active research
- A soft pulsing ring expands from recently unlocked nodes.
- Background is deep navy/black with a subtle starfield effect.
- **The `♦`, `?`, `!` are iconic node shapes, not ASCII fallback.** They
  are part of the constellation visual language.

**Implementation extension required:**
- Star points and dotted lines could be vector drawings rather than text.
- Background animation (starfield) would require a custom shader or many
  tiny UI nodes.

### 4.8 Docking — Blueprint Drawing

**Visual metaphor:** Blueprint/engineering schematic with dashed lines and
annotations.

**How it should look:**
- White/cyan lines on a blue background (blueprint paper).
- Ship outlines are drawn as dashed lines.
- Docking ports are circles with arrows.
- Umbilical connections are thick double lines with `DOCKING` labels.
- Measurement-like annotations (e.g., `2.0m`) appear near ports.
- Air pressure locked/closed states shown with red crosses.
- **This layer does not use ASCII `--`, `|`, `=` symbols.** It is vector
  blueprint drawing.

**Implementation extension required:**
- Custom vector drawing for dashed lines, circles, arrows, and annotations.
- A blueprint background texture/solid color is easy.

### 4.9 Sensors — Radar Sweep Overlay

**Visual metaphor:** Radar scope / sonar screen with rotating sweep line.

**How it looks:**
- The map is drawn as a dark circle (or rectangle) with a **rotating sweep
  line**.
- Detected objects appear as bright blips that fade over time.
- Own drones appear as steady green markers.
- Sweep updates have a subtle phosphor trail.
- A compass rose may be shown.
- **No ASCII `~` or `+` sweep marks.** The sweep line and pings are drawn
  graphically.

**Implementation extension required:**
- Needs a custom overlay that can draw a rotating line and blips.
- Could temporarily use simplified vector primitives, but not ASCII.

### 4.10 Reserved — Static / Unknown Fog

**Mode:** Static noise / placeholder.

- No real data. Show a screen of random `?`, `#`, or static-like noise.
- Color is gray on black, with occasional flicker.
- **This is not ASCII fallback; it is a placeholder visual for an
  unimplemented layer.** The `?` and `#` are noise glyphs, not structural
  map characters.
- This reinforces the idea that the player does not yet have access to this
  layer.

---

## 5. Transition Strategy

Given current constraints, we cannot implement all modes at once. We propose
a two-phase approach:

### Phase 1 — Distinct Provisional Modes (Not ASCII)

- Keep the single string renderer only for the structural layer.
- For every other layer, implement a **provisional distinct renderer** that
  does **not use ASCII glyphs from the structural layer**.
- These provisional modes can be simple, low-resolution, or stylized, but
  they must be visually unique:
  - Electrical: thin colored line segments and circles using UI `NodeBundle` or custom mesh.
  - Fluidics: colored filled tiles/rectangles arranged as low-res pixels.
  - Atmospherics: per-cell background colors representing pressure bands.
  - Thermal: continuous color ramp over cells.
  - Computer: existing terminal text (canonical for this layer).
  - Research: small vector dots and dotted lines.
  - Docking: vector dashed lines on a blue background.
  - Sensors: a simple radar sweep using a rotating UI overlay.
- No layer may fall back to `#`, `.`, `+`, `~`, or any other structural glyph.
- Structural ASCII remains untouched.

### Phase 2 — True Multi-Mode Rendering

- Introduce a `SchematicRenderMode` enum.
- Each mode has its own rendering system and UI entity tree.
- Reuse existing ASCII only for structural.
- Add:
  - A `VectorOverlay` component for electrical/docking/research.
  - A `PixelBuffer` component for fluidics.
  - A `HeatmapLayer` for thermal/atmospherics.
  - A `TerminalLayer` for computer.
  - A `RadarScope` for sensors.
- These modes can be toggled independently and will eventually replace the
  provisional modes entirely for their layers.

---

## 6. Needed Technical Foundations

To support the multi-mode vision, the following Bevy-side systems are
required:

1. **Per-cell color support** — either a grid of `Text` sections or colored
   `NodeBundle` tiles.
2. **Custom vector rendering** — use `bevy_prototype_lyon` or procedural
   meshes for lines, circles, arrows, and dashed outlines.
3. **Pixel-buffer rendering** — a CPU-side array turned into an `Image` /
   `TextureAtlas` via `Assets<Image>` for fluidics.
4. **Heatmap shader** — a simple fragment shader able to map a scalar field
   to a gradient.
5. **Animation timing** — deterministic `Timer` resources for pulse, sweep,
   flow, and static effects.
6. **Blueprint/terminal styling** — the UI already supports `Text` and
   `NodeBundle`, so these are low-cost.

These are substantial but align with the overall architecture’s plan to keep
rendering diegetic and layered.

---

## 7. Animation Philosophy

Animations should be:

- **Slow and legible** — no rapid flashing.
- **Deterministic** — driven by fixed timers, not frame-time, for testability.
- **Purposeful** — only animate states that change meaningfully:
  - Electrical pulse only when powered.
  - Fluid flow only when pump active.
  - Pressure contours only when atmosphere simulation updates.
  - Radar sweep always, but fade blips based on age.
  - Research pulse on recently unlocked nodes.

---

## 8. Visual Distinction and Accessibility

Every layer must be **entirely distinct** from the structural layer and from
each other. The structural layer is the only ASCII layer. No other layer
should use `#`, `.`, `+`, `~`, `=`, or other structural glyphs as part of
its visual representation.

Instead of providing ASCII fallbacks, each mode must be designed from the
start with clear, colorblind-safe visual differences:

- **Shape:** vector lines, pixel blocks, heatmap gradients, terminal text, radial sweep, etc.
- **Color:** each mode has a dominant palette that is not solely dependent on hue; pair colors with patterns or motion where possible.
- **Motion:** electrical pulse, fluid ripple, thermal bloom, radar sweep, research pulse. Motion can help distinguish layers even when color is absent.

A global option may allow the player to hide all layers except structural, but
that is not a degradation to ASCII; it is a view selection. System layers
remain visually distinct in their own modes at all times.

---

## 9. Conclusion

This ambitious multi-mode approach turns the schematic map into a set of
distinct, diegetic instrument views. The structural layer remains the
familiar roguelike map, while each system layer becomes its own small
visual simulation. The technical roadmap is large but can be phased, and the
architecture of isolated layer renderers will keep the code modular and
testable.
