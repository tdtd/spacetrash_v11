use bevy::prelude::*;

use crate::components::*;
use crate::resources::scenario::ShipTemplateLibrary;
use crate::resources::FocusedEntity;
use crate::systems::ship_generation::spawn_ship_from_template;

/// Spawns the initial game world: a light, a simple ship (one room),
/// and a player cube. The camera is already created at startup.
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    template_library: Res<ShipTemplateLibrary>,
    mut focused_entity: ResMut<FocusedEntity>,
) {
    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameplayEntity,
    ));

    // Get default template
    let template = template_library
        .templates
        .get("test_ship")
        .expect("Default ship template not found")
        .clone();

    // Spawn ship using generator
    let _ship = spawn_ship_from_template(
        &mut commands,
        &mut meshes,
        &mut materials,
        &template,
        Transform::default(),
    );

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

    // Set initial focus to the Blackbox.
    focused_entity.entity = Some(player_entity);
}
