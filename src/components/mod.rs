use bevy::prelude::*;

use crate::procgen::tile::{TileType, WallDirection};

/// Velocity vector in world units per second.
#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

/// Current yaw rotation (around the Y axis) in radians.
#[derive(Component, Default)]
pub struct Rotation {
    pub yaw: f32,
}

/// Angular velocity around the Y axis, in radians per second.
#[derive(Component, Default)]
pub struct AngularVelocity {
    pub yaw: f32,
}

/// Axis-aligned bounding box collider, defined by half-extents in world units.
#[derive(Component)]
pub struct AabbCollider {
    pub half_extents: Vec3,
}

/// Circle collider defined by radius in world units.
#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32,
}

/// Actor size determines default collider behavior and movement constraints.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorSize {
    Small,
    Standard,
    Large,
}

impl ActorSize {
    /// Returns the standard collider radius for this size, if any.
    ///
    /// `Small` has no collision.
    pub fn collider_radius(&self) -> Option<f32> {
        match self {
            ActorSize::Small => None,
            ActorSize::Standard => Some(1.0 / 6.0),
            ActorSize::Large => Some(1.0 / 3.0),
        }
    }
}

/// Physical properties for a dynamic actor.
#[derive(Component, Clone, Debug)]
pub struct PhysicsBody {
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub max_linear_speed: f32,
    pub max_angular_speed: f32,
}

/// Marks the main player-controlled character.
#[derive(Component, Default)]
pub struct Player;

/// Marks an entity as a controllable drone/actor unit.
#[derive(Component, Default)]
pub struct Controllable;

/// Marks an entity as the primary drone, the "Blackbox".
#[derive(Component, Default)]
pub struct Blackbox;

/// Marks an entity as a drone and stores its unique index (1-9).
#[derive(Component)]
pub struct Drone {
    pub index: u8,
}

/// First-person camera definition for drones and the Blackbox.
///
/// The world camera is placed at `entity_transform.translation + entity_transform.rotation * eye_offset`
/// and adopts the entity's rotation. The perspective projection FOV is taken from `fov` (radians).
#[derive(Component)]
pub struct FirstPersonCamera {
    pub eye_offset: Vec3,
    pub fov: f32,
}

/// Marker for the active world camera used for first-person and map views.
#[derive(Component, Default)]
pub struct ActiveWorldCamera;

/// Marks an entity as a ship (or part of a ship).
#[derive(Component, Default)]
pub struct Ship;

/// Marks an entity as a room within a ship grid.
#[derive(Component)]
pub struct Room {
    pub width: f32,
    pub height: f32,
    pub ship_entity: Option<Entity>,
}

impl Default for Room {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            ship_entity: None,
        }
    }
}

/// A docking port on a ship.
#[derive(Component)]
pub struct DockingPort {
    pub ship_entity: Entity,
    pub local_position: Vec3,
    pub connected_to: Option<Entity>, // other DockingPort entity
}

impl DockingPort {
    pub fn new(ship_entity: Entity, local_position: Vec3) -> Self {
        Self {
            ship_entity,
            local_position,
            connected_to: None,
        }
    }
}

/// For future use: grid coordinates of a room within a ship.
#[derive(Component)]
pub struct GridCoordinates {
    pub x: i32,
    pub y: i32,
}

/// A single inward-facing wall segment.
///
/// `a` and `b` are local-space endpoints in tile XZ coordinates.
/// `normal` points inward, away from the wall and into the room.
#[derive(Clone, Debug)]
pub struct WallSegment {
    pub a: Vec2,
    pub b: Vec2,
    pub normal: Vec2,
}

/// Collider made of inward-facing wall segments.
///
/// This replaces full AABB colliders on directional room walls.
#[derive(Component, Clone, Debug)]
pub struct WallSegmentCollider {
    pub segments: Vec<WallSegment>,
}

impl WallSegmentCollider {
    /// Builds the correct collider segments for an 8-direction wall tile.
    pub fn for_direction(direction: WallDirection) -> Self {
        match direction {
            WallDirection::North => Self {
                segments: vec![WallSegment {
                    a: Vec2::new(-0.5, -0.5),
                    b: Vec2::new(0.5, -0.5),
                    normal: Vec2::new(0.0, -1.0),
                }],
            },
            WallDirection::South => Self {
                segments: vec![WallSegment {
                    a: Vec2::new(-0.5, 0.5),
                    b: Vec2::new(0.5, 0.5),
                    normal: Vec2::new(0.0, 1.0),
                }],
            },
            WallDirection::East => Self {
                segments: vec![WallSegment {
                    a: Vec2::new(-0.5, -0.5),
                    b: Vec2::new(-0.5, 0.5),
                    normal: Vec2::new(-1.0, 0.0),
                }],
            },
            WallDirection::West => Self {
                segments: vec![WallSegment {
                    a: Vec2::new(0.5, -0.5),
                    b: Vec2::new(0.5, 0.5),
                    normal: Vec2::new(1.0, 0.0),
                }],
            },
            WallDirection::NorthEast => Self {
                segments: vec![
                    WallSegment {
                        a: Vec2::new(-0.5, -0.5),
                        b: Vec2::new(-0.5, 0.5),
                        normal: Vec2::new(-1.0, 0.0),
                    },
                    WallSegment {
                        a: Vec2::new(-0.5, -0.5),
                        b: Vec2::new(0.5, -0.5),
                        normal: Vec2::new(0.0, -1.0),
                    },
                ],
            },
            WallDirection::NorthWest => Self {
                segments: vec![
                    WallSegment {
                        a: Vec2::new(0.5, -0.5),
                        b: Vec2::new(0.5, 0.5),
                        normal: Vec2::new(1.0, 0.0),
                    },
                    WallSegment {
                        a: Vec2::new(-0.5, -0.5),
                        b: Vec2::new(0.5, -0.5),
                        normal: Vec2::new(0.0, -1.0),
                    },
                ],
            },
            WallDirection::SouthEast => Self {
                segments: vec![
                    WallSegment {
                        a: Vec2::new(-0.5, -0.5),
                        b: Vec2::new(-0.5, 0.5),
                        normal: Vec2::new(-1.0, 0.0),
                    },
                    WallSegment {
                        a: Vec2::new(-0.5, 0.5),
                        b: Vec2::new(0.5, 0.5),
                        normal: Vec2::new(0.0, 1.0),
                    },
                ],
            },
            WallDirection::SouthWest => Self {
                segments: vec![
                    WallSegment {
                        a: Vec2::new(0.5, -0.5),
                        b: Vec2::new(0.5, 0.5),
                        normal: Vec2::new(1.0, 0.0),
                    },
                    WallSegment {
                        a: Vec2::new(-0.5, 0.5),
                        b: Vec2::new(0.5, 0.5),
                        normal: Vec2::new(0.0, 1.0),
                    },
                ],
            },
        }
    }
}

/// Component representing a generated tile node in the ship grid.
///
/// Stores the tile's type so the schematic map can render the correct glyph.
#[derive(Component, Clone, Debug)]
pub struct TileNode {
    pub tile_type: TileType,
}

/// Marker component for entities that belong to the active gameplay world and
/// should be despawned when exiting the `Playing` state.
#[derive(Component, Default)]
pub struct GameplayEntity;
