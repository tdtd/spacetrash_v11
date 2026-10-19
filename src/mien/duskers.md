# Duskers Comparison Guide

This document tracks how Spacetrash v11 is modeled after the original game **Duskers**, and where we intentionally diverge.

## Original Game Reference

Duskers is a top-down, real-time salvage and exploration game where the player controls a small squad of drones via a command-line-style interface. The player explores derelict spaceships, scavenges resources, avoids hazards, and fights hostile lifeforms.

## Drone/Party Limit

- **Original Duskers:** The player could control up to 4 drones at a time.
- **Spacetrash v11:** The player can control up to 9 drones/actor units at a time.

This increase creates a wider tactical space, but also requires more careful UI and AI handling. The architecture must support selecting and commanding up to 9 units without overwhelming the player.

## Enemy Species

### Original Duskers

Duskers had 4 enemy types:

- Killbots
- Slime
- Leapers
- Hornets

Each species had distinct but fixed features and behaviors. They did not interact with each other in meaningful ways.

### Spacetrash v11

We do not want only fixed species. Instead, enemies are generated as modular actors from upgrade components, as described in the architecture and mission docs. The goal is to create more complex, interacting agents whose behavior emerges from combinations of sensory, effector, and behavior upgrades.

Potential interaction examples:

- Gather information from other enemies (shared perception)
- React to each other's attacks or deaths
- Coordinate movement patterns
- Use one unit as a scout and another as a guard

This moves the game beyond fixed species and toward procedurally generated, emergent behaviors.

## Single Ray Instrument Constraint

The original Duskers had limited sensor/weapon options; each drone could only carry a single tool at a time. Spacetrash follows a similar principle but more strictly: **each agent may have at most one active ray-based instrument**.

A ray instrument can be:

- A visible-light camera
- A LIDAR
- A turret gun that uses ray-based aiming
- A thermal imager
- A radar emitter/receiver

A visible-light camera is not a separate two-ray system; it is simply a light-ray-detecting 3D camera. This simplifies the camera greatly and prevents expensive multi-ray operations.

This constraint applies to both player drones and generated enemies. It ensures a consistent performance baseline and a predictable sensor economy.

## Mapping Existing Duskers Mechanics to Spacetrash Concepts

- **Drones as actors:** Duskers drones map to actor entities whose chassis and upgrades are modular.
- **Ship exploration:** Duskers derelict ships map to procedurally generated rectangular room grids with docking.
- **Command line interface:** While Spacetrash v11 currently uses direct input for prototype, the long-term design may include a similar diegetic command interface for group control.
- **Resource scavenging:** Both games focus on returning resources to the player's base/ship.

## Intentional Differences

- Drone limit increased from 4 to 9.
- Enemy species are not limited to fixed types; instead we generate enemies from a shared pool of upgrade modules.
- Enemies and actors can carry sensors such as cameras, lights, microphones, and radars, reflecting the ray-instrument system.
- Every agent is limited to one active ray instrument at a time.
- A visible-light camera is simply a light-ray-detecting 3D camera, not part of a separate two-ray system.
- Room interiors are planned to be populated by Wave Function Collapse, creating more varied layouts than simple rectangles.
- The game uses continuous physics rather than grid-based discrete movement.
- Docking between ships merges playfields, creating a dynamic world that grows during play.
- Original Duskers had only a small number of item types and special rooms. Spacetrash will introduce many new room types and resources, creating more emergent interactions.
- Original Duskers had only a few special rooms that did not interact in any meaningful way. Spacetrash treats ships as complex machines with interdependent systems (electrical, computer, fluidics, atmospherics, thermals) that must be kept in relative balance, or failures will cascade. This is inspired by FTL: Faster Than Light's power-management crises, but adapted to a salvage/exploration context.
- Original Duskers kept salvage actions separate from ship system stability. In Spacetrash, stripping valuable components (e.g., warp core, computer, radiator, oxygen reserves) actively triggers cascading failures, making every salvage operation a risk-reward decision.
- Original Duskers did not include a fluidics network. Spacetrash introduces reservoirs, pipes, valves, and pumps, allowing the player to transfer fluids between rooms and ships through the docking umbilicus. Fluid transfer requires player control of all rooms along the route. Fluid types include oxygen, nitrogen, hydrogen, hydrazine, rocket fuel, coolant, water, and others.
- Original Duskers had only a simple atmospherics model, room-by-room vacuum and radiation. Spacetrash plans a low-frequency compartment-based gas mixing simulation for pressure, oxygen, contaminants, and interactions with life support, breaches, and salvage actions.
- Original Duskers had actors of only one size. Spacetrash introduces three actor sizes: Small (can enter air vents, no collision), Standard (requires 1/3 tile), and Large (requires 2/3 tile). This affects movement, door passage, and combat.
- Original Duskers had limited travel between ships, relying mostly on docking. Spacetrash adds specific travel methods: main airlock umbilicus (2-tile bridge), shuttles (mini ships), and teleporters (expensive). These provide additional strategic options.
- Original Duskers did not have a hidden tech tree or research system. Spacetrash introduces a hidden, diegetically represented research mechanic inspired by Factorio, where the player gradually unlocks mechanics, upgrades, room types, and ship systems through in-fiction research actions, avoiding cognitive overload.
- Original Duskers did not have a **live fog of war**; its explored map remained visible. Spacetrash uses a live fog of war where visibility depends on the current positions and capacities of agents, not simply on exploration history.

This file is a living reference. Update it when another intentional difference or expansion is decided.
