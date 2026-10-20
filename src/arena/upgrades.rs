use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{
    AggressiveBehavior, AngularVelocity, CircleCollider, FleeBehavior, FollowLeaderBehavior,
    GameplayEntity, GuardBehavior, LidarSensor, MeleeWeapon, PatrolBehavior, PhysicsBody, RadarSensor,
    RayInstrument, Rotation, ScavengeBehavior, ThermalSensor, TurretWeapon, Velocity, VisibleCamera,
};

/// Broad category for balance and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeCategory {
    Movement,
    Sensor,
    Effector,
    Defensive,
    Utility,
    Behavior,
}

/// Physical base of an actor before upgrades are applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ChassisDefinition {
    pub id: String,
    pub base_radius: f32,
    pub base_linear_damping: f32,
    pub base_angular_damping: f32,
    pub base_max_linear_speed: f32,
    pub base_max_angular_speed: f32,
}

/// Movement values that override chassis defaults when a movement upgrade
/// is installed.
#[derive(Debug, Clone, PartialEq)]
pub struct MovementProfile {
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub max_linear_speed: f32,
    pub max_angular_speed: f32,
}

/// Ray-based or non-ray sensor types currently supported by the assembler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorKind {
    None,
    VisibleCamera,
    Thermal,
    Radar,
    Lidar,
}

/// Weapon types currently supported by the assembler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    None,
    Melee,
    Turret,
}

/// Behavior upgrade types currently supported by the assembler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorKind {
    None,
    Patrol,
    Guard,
    Aggressive,
    Flee,
    Scavenge,
    FollowLeader,
}

/// A single upgrade module definition.
#[derive(Debug, Clone, PartialEq)]
pub struct UpgradeDefinition {
    pub id: String,
    pub category: UpgradeCategory,
    pub ray_instrument: bool,
    pub movement: Option<MovementProfile>,
    pub sensor: SensorKind,
    pub weapon: WeaponKind,
    pub behavior: BehaviorKind,
}

/// A complete actor build description.
#[derive(Debug, Clone, PartialEq)]
pub struct ActorBuildSpec {
    pub chassis: String,
    pub upgrades: Vec<String>,
}

/// Central registry of available chassis and upgrades.
#[derive(Resource, Default)]
pub struct UpgradeRegistry {
    pub chassis: HashMap<String, ChassisDefinition>,
    pub upgrades: HashMap<String, UpgradeDefinition>,
}

impl UpgradeRegistry {
    pub fn insert_chassis(&mut self, def: ChassisDefinition) {
        self.chassis.insert(def.id.clone(), def);
    }

    pub fn insert_upgrade(&mut self, def: UpgradeDefinition) {
        self.upgrades.insert(def.id.clone(), def);
    }

    pub fn get_chassis(&self, id: &str) -> Option<&ChassisDefinition> {
        self.chassis.get(id)
    }

    pub fn get_upgrade(&self, id: &str) -> Option<&UpgradeDefinition> {
        self.upgrades.get(id)
    }
}

/// Builds an actor entity from a build spec.
///
/// Enforces the single active ray instrument constraint and returns an error
/// for unknown IDs, duplicate movement upgrades, or invalid ray counts.
///
/// Spawned entities are marked with `GameplayEntity`, so they can be cleaned
/// up with the standard world reset.
pub fn build_actor(
    world: &mut World,
    spec: &ActorBuildSpec,
    registry: &UpgradeRegistry,
) -> Result<Entity, String> {
    let chassis = registry
        .get_chassis(&spec.chassis)
        .ok_or_else(|| format!("unknown chassis id: {}", spec.chassis))?;

    let mut ray_count = 0usize;
    let mut movement = None;
    let mut sensor = SensorKind::None;
    let mut weapon = WeaponKind::None;
    let mut behavior = BehaviorKind::None;

    for upgrade_id in &spec.upgrades {
        let upgrade = registry
            .get_upgrade(upgrade_id)
            .ok_or_else(|| format!("unknown upgrade id: {}", upgrade_id))?;

        if upgrade.ray_instrument {
            ray_count += 1;
        }

        if upgrade.movement.is_some() {
            if movement.is_some() {
                return Err("actor has more than one movement upgrade".to_string());
            }
            movement = upgrade.movement.clone();
        }

        if upgrade.sensor != SensorKind::None {
            sensor = upgrade.sensor;
        }

        if upgrade.weapon != WeaponKind::None {
            weapon = upgrade.weapon;
        }

        if upgrade.behavior != BehaviorKind::None {
            behavior = upgrade.behavior;
        }
    }

    if ray_count > 1 {
        return Err(format!(
            "actor has {ray_count} active ray instruments; maximum is 1"
        ));
    }

    let mut entity = world.spawn((
        Transform::default(),
        Velocity::default(),
        Rotation::default(),
        AngularVelocity::default(),
        CircleCollider {
            radius: chassis.base_radius,
        },
        PhysicsBody {
            linear_damping: movement
                .as_ref()
                .map(|m| m.linear_damping)
                .unwrap_or(chassis.base_linear_damping),
            angular_damping: movement
                .as_ref()
                .map(|m| m.angular_damping)
                .unwrap_or(chassis.base_angular_damping),
            max_linear_speed: movement
                .as_ref()
                .map(|m| m.max_linear_speed)
                .unwrap_or(chassis.base_max_linear_speed),
            max_angular_speed: movement
                .as_ref()
                .map(|m| m.max_angular_speed)
                .unwrap_or(chassis.base_max_angular_speed),
        },
        GameplayEntity,
    ));

    match sensor {
        SensorKind::VisibleCamera => {
            entity.insert(VisibleCamera);
            entity.insert(RayInstrument);
        }
        SensorKind::Thermal => {
            entity.insert(ThermalSensor);
            entity.insert(RayInstrument);
        }
        SensorKind::Radar => {
            entity.insert(RadarSensor);
            entity.insert(RayInstrument);
        }
        SensorKind::Lidar => {
            entity.insert(LidarSensor);
            entity.insert(RayInstrument);
        }
        SensorKind::None => {}
    }

    match weapon {
        WeaponKind::Melee => {
            entity.insert(MeleeWeapon);
        }
        WeaponKind::Turret => {
            entity.insert(TurretWeapon);
            entity.insert(RayInstrument);
        }
        WeaponKind::None => {}
    }

    match behavior {
        BehaviorKind::Patrol => {
            entity.insert(PatrolBehavior);
        }
        BehaviorKind::Guard => {
            entity.insert(GuardBehavior);
        }
        BehaviorKind::Aggressive => {
            entity.insert(AggressiveBehavior);
        }
        BehaviorKind::Flee => {
            entity.insert(FleeBehavior);
        }
        BehaviorKind::Scavenge => {
            entity.insert(ScavengeBehavior);
        }
        BehaviorKind::FollowLeader => {
            entity.insert(FollowLeaderBehavior);
        }
        BehaviorKind::None => {}
    }

    Ok(entity.id())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> UpgradeRegistry {
        let mut registry = UpgradeRegistry::default();

        registry.insert_chassis(ChassisDefinition {
            id: "light_drone".to_string(),
            base_radius: 0.1667,
            base_linear_damping: 10.0,
            base_angular_damping: 6.0,
            base_max_linear_speed: 4.0,
            base_max_angular_speed: 8.0,
        });

        registry.insert_upgrade(UpgradeDefinition {
            id: "visible_camera".to_string(),
            category: UpgradeCategory::Sensor,
            ray_instrument: true,
            movement: None,
            sensor: SensorKind::VisibleCamera,
            weapon: WeaponKind::None,
            behavior: BehaviorKind::None,
        });

        registry.insert_upgrade(UpgradeDefinition {
            id: "thermal_sensor".to_string(),
            category: UpgradeCategory::Sensor,
            ray_instrument: true,
            movement: None,
            sensor: SensorKind::Thermal,
            weapon: WeaponKind::None,
            behavior: BehaviorKind::None,
        });

        registry.insert_upgrade(UpgradeDefinition {
            id: "hover_drive".to_string(),
            category: UpgradeCategory::Movement,
            ray_instrument: false,
            movement: Some(MovementProfile {
                linear_damping: 4.0,
                angular_damping: 3.0,
                max_linear_speed: 6.0,
                max_angular_speed: 10.0,
            }),
            sensor: SensorKind::None,
            weapon: WeaponKind::None,
            behavior: BehaviorKind::None,
        });

        registry.insert_upgrade(UpgradeDefinition {
            id: "patrol".to_string(),
            category: UpgradeCategory::Behavior,
            ray_instrument: false,
            movement: None,
            sensor: SensorKind::None,
            weapon: WeaponKind::None,
            behavior: BehaviorKind::Patrol,
        });

        registry
    }

    #[test]
    fn builds_actor_with_visible_camera_and_patrol() {
        let registry = test_registry();
        let spec = ActorBuildSpec {
            chassis: "light_drone".to_string(),
            upgrades: vec!["visible_camera".to_string(), "patrol".to_string()],
        };

        let mut world = World::new();
        let entity = build_actor(&mut world, &spec, &registry).unwrap();

        assert!(world.get::<VisibleCamera>(entity).is_some());
        assert!(world.get::<RayInstrument>(entity).is_some());
        assert!(world.get::<PatrolBehavior>(entity).is_some());
        assert!(world.get::<ThermalSensor>(entity).is_none());
    }

    #[test]
    fn rejects_two_active_ray_instruments() {
        let registry = test_registry();
        let spec = ActorBuildSpec {
            chassis: "light_drone".to_string(),
            upgrades: vec![
                "visible_camera".to_string(),
                "thermal_sensor".to_string(),
            ],
        };

        let mut world = World::new();
        let result = build_actor(&mut world, &spec, &registry);

        assert!(result.is_err());
    }

    #[test]
    fn applies_movement_profile() {
        let registry = test_registry();
        let spec = ActorBuildSpec {
            chassis: "light_drone".to_string(),
            upgrades: vec!["hover_drive".to_string()],
        };

        let mut world = World::new();
        let entity = build_actor(&mut world, &spec, &registry).unwrap();

        let body = world.get::<PhysicsBody>(entity).unwrap();
        assert_eq!(body.linear_damping, 4.0);
        assert_eq!(body.max_linear_speed, 6.0);
    }

    #[test]
    fn actor_build_is_deterministic() {
        let registry = test_registry();
        let spec = ActorBuildSpec {
            chassis: "light_drone".to_string(),
            upgrades: vec!["visible_camera".to_string(), "patrol".to_string()],
        };

        let mut world1 = World::new();
        let entity1 = build_actor(&mut world1, &spec, &registry).unwrap();

        let mut world2 = World::new();
        let entity2 = build_actor(&mut world2, &spec, &registry).unwrap();

        assert_eq!(
            world1.get::<VisibleCamera>(entity1).is_some(),
            world2.get::<VisibleCamera>(entity2).is_some()
        );
        assert_eq!(
            world1.get::<PatrolBehavior>(entity1).is_some(),
            world2.get::<PatrolBehavior>(entity2).is_some()
        );
        assert_eq!(
            world1.get::<CircleCollider>(entity1).unwrap().radius,
            world2.get::<CircleCollider>(entity2).unwrap().radius
        );
    }
}
