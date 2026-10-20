use bevy::prelude::*;

use crate::components::*;
use crate::resources::scenario::{ActiveScenario, ShipTemplateLibrary};
use crate::resources::FocusedEntity;
use crate::systems::ship_generation::spawn_ship_from_template;

/// Spawns the initial game world from the active scenario.
///
/// If no scenario is active, this falls back to the built-in default
/// scenario so the main menu `Start Game` path continues to work.
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    template_library: Res<ShipTemplateLibrary>,
    active_scenario: Res<ActiveScenario>,
    mut focused_entity: ResMut<FocusedEntity>,
) {
    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameplayEntity,
    ));

    let scenario = active_scenario.scenario.clone().unwrap_or_else(crate::resources::scenario::default_scenario);

    // Spawn each ship from the scenario.
    for ship_instance in &scenario.ships {
        let Some(template) = template_library.templates.get(&ship_instance.template_id) else {
            warn!(
                "Scenario references unknown template id: {}",
                ship_instance.template_id
            );
            continue;
        };

        let template = template.clone();
        let world_transform = Transform::from_xyz(
            ship_instance.position[0],
            0.0,
            ship_instance.position[1],
        )
        .with_rotation(Quat::from_rotation_y(ship_instance.rotation));

        let _ship = spawn_ship_from_template(
            &mut commands,
            &mut meshes,
            &mut materials,
            &template,
            world_transform,
        );
    }

    // Player entity (the Blackbox main character)
    let player_entity = commands
        .spawn((
            Player,
            Blackbox,
            Controllable,
            Velocity::default(),
            Rotation::default(),
            AngularVelocity::default(),
            CircleCollider { radius: 0.1667 },
            FirstPersonCamera {
                eye_offset: Vec3::new(0.0, 0.6, 0.0),
                fov: 70.0_f32.to_radians(),
            },
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5))),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.2, 0.6, 1.0),
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 0.25, 0.0),
                ..default()
            },
            GameplayEntity,
        ))
        .id();

    // Spawn a few test drones for focus switching.
    // In the future, drone configurations will come from ship templates/procgen.
    for i in 1..=3u8 {
        commands.spawn((
            Drone { index: i },
            Controllable,
            Velocity::default(),
            Rotation::default(),
            AngularVelocity::default(),
            CircleCollider { radius: 0.1667 },
            FirstPersonCamera {
                eye_offset: Vec3::new(0.0, 0.5, 0.0),
                fov: 70.0_f32.to_radians(),
            },
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5))),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.8, 0.4, 0.1),
                    ..default()
                }),
                transform: Transform::from_xyz(i as f32 * 2.0, 0.25, 0.0),
                ..default()
            },
            GameplayEntity,
        ));
    }

    // Spawn a few enemy agents with predetermined behavior for now.
    //
    // They are marked with `Enemy`, are not `Controllable`, and have both a
    // `BehaviorDirective` and matching `AiBehaviorState`. Their movement is
    // driven by `behavior_movement_system` before the physics step.
    let enemy_mesh = meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5)));
    let enemy_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.9, 0.2, 0.2),
        ..default()
    });

    let enemy_states = [
        (AiBehaviorState::Patrol, BehaviorDirective::Patrol, Vec3::new(-4.0, 0.25, 2.0)),
        (AiBehaviorState::Aggressive, BehaviorDirective::Aggressive, Vec3::new(4.0, 0.25, -2.0)),
        (AiBehaviorState::Scavenge, BehaviorDirective::Scavenge, Vec3::new(-2.0, 0.25, -4.0)),
    ];

    for (ai_state, directive, pos) in enemy_states {
        commands.spawn((
            Enemy,
            ai_state,
            directive,
            Velocity::default(),
            Rotation { yaw: 0.0 },
            AngularVelocity::default(),
            CircleCollider { radius: 0.1667 },
            PbrBundle {
                mesh: enemy_mesh.clone(),
                material: enemy_material.clone(),
                transform: Transform::from_translation(pos),
                ..default()
            },
            GameplayEntity,
        ));
    }

    // Set initial focus to the Blackbox.
    focused_entity.entity = Some(player_entity);
}
