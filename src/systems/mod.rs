use bevy::prelude::*;
use bevy::render::camera::Projection;
use std::collections::HashMap;

use crate::components::*;
use crate::resources::*;

pub mod collision;

pub mod set_pieces;

pub mod behavior;
pub use behavior::{behavior_arbitration_system, behavior_movement_system};

pub mod spectator;

pub mod terminal;
pub use terminal::{parse_terminal_command, terminal_input_system};

mod setup_scene;
pub use setup_scene::setup_scene;

mod ship_generation;

/// Reads keyboard input and updates the currently focused entity's velocity using acceleration.
pub fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    focused_entity: Res<FocusedEntity>,
    mut query: Query<(&mut Velocity, &mut AngularVelocity, Option<&Rotation>)>,
    config: Res<PhysicsConfig>,
    speed: Res<SimulationSpeed>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds() * speed.scale;

    // Forward/back from W/S, relative to camera facing.
    let mut forward = 0.0;
    if keyboard.pressed(KeyCode::KeyW) {
        forward += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        forward -= 1.0;
    }

    // Strafing from A/D, relative to camera facing.
    let mut strafe = 0.0;
    if keyboard.pressed(KeyCode::KeyA) {
        strafe -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        strafe += 1.0;
    }

    // Rotation input: Q/Z rotate left, E/C rotate right.
    let mut rot_input = 0.0;
    if keyboard.pressed(KeyCode::KeyQ) || keyboard.pressed(KeyCode::KeyZ) {
        rot_input += 1.0;   // left turn is positive yaw
    }
    if keyboard.pressed(KeyCode::KeyE) || keyboard.pressed(KeyCode::KeyC) {
        rot_input -= 1.0;   // right turn is negative yaw
    }

    // Local movement: forward is negative Z (camera forward), strafe is X.
    let local_linear = Vec3::new(strafe, 0.0, -forward);
    let local_linear = if local_linear.length_squared() > 0.0 {
        local_linear.normalize()
    } else {
        Vec3::ZERO
    };

    if let Some(entity) = focused_entity.entity {
        if let Ok((mut velocity, mut angular_velocity, rotation_opt)) = query.get_mut(entity) {
            // Rotate local movement into world space using standard Y-rotation matrix.
            let yaw = rotation_opt.map(|r| r.yaw).unwrap_or(0.0);
            let cos_yaw = yaw.cos();
            let sin_yaw = yaw.sin();

            let world_linear = Vec3::new(
                local_linear.x * cos_yaw + local_linear.z * sin_yaw,
                0.0,
                -local_linear.x * sin_yaw + local_linear.z * cos_yaw,
            );

            // Apply linear acceleration.
            let mut new_velocity = velocity.0 + world_linear * config.acceleration * dt;
            if new_velocity.length() > config.max_speed {
                new_velocity = new_velocity.normalize() * config.max_speed;
            }
            velocity.0 = new_velocity;

            // Apply angular acceleration.
            let mut new_ang_vel = angular_velocity.yaw + rot_input * config.angular_acceleration * dt;
            if new_ang_vel.abs() > config.angular_max_speed {
                new_ang_vel = new_ang_vel.signum() * config.angular_max_speed;
            }
            angular_velocity.yaw = new_ang_vel;
        }
    }
}

/// Applies damping, integrates velocity into Transform, and resolves collisions
/// against static AABB colliders, inward-facing wall segments, and other
/// dynamic circle colliders.
pub fn movement_system(
    time: Res<Time>,
    speed: Res<SimulationSpeed>,
    config: Res<PhysicsConfig>,
    mut dynamic_query: Query<(
        Entity,
        &mut Transform,
        &mut Velocity,
        &mut Rotation,
        &mut AngularVelocity,
        &CircleCollider,
        Option<&PhysicsBody>,
        Option<&ActorSize>,
    )>,
    static_query: Query<(&Transform, &AabbCollider), Without<Velocity>>,
    wall_static_query: Query<(&Transform, &WallSegmentCollider), Without<Velocity>>,
) {
    let dt = time.delta_seconds() * speed.scale;

    // Phase 1: integrate each dynamic actor and resolve static collisions.
    let mut resolved_positions: Vec<(Entity, Vec3, f32, bool)> = Vec::new();

    for (entity, mut transform, mut velocity, mut rotation, mut angular_velocity, collider, physics_body, actor_size) in dynamic_query.iter_mut() {
        // Use per-actor physics body if present, otherwise global config.
        let linear_damping = physics_body.map(|b| b.linear_damping).unwrap_or(config.damping);
        let angular_damping = physics_body.map(|b| b.angular_damping).unwrap_or(config.angular_damping);
        let max_linear_speed = physics_body.map(|b| b.max_linear_speed).unwrap_or(config.max_speed);
        let max_angular_speed = physics_body.map(|b| b.max_angular_speed).unwrap_or(config.angular_max_speed);

        // Linear damping.
        let damping_factor = (1.0 - linear_damping * dt).max(0.0);
        velocity.0 *= damping_factor;
        // Clamp linear speed.
        if velocity.0.length() > max_linear_speed {
            velocity.0 = velocity.0.normalize() * max_linear_speed;
        }

        // Angular damping.
        let angular_damping_factor = (1.0 - angular_damping * dt).max(0.0);
        angular_velocity.yaw *= angular_damping_factor;
        // Clamp angular velocity.
        if angular_velocity.yaw.abs() > max_angular_speed {
            angular_velocity.yaw = angular_velocity.yaw.signum() * max_angular_speed;
        }

        // Integrate angle from angular velocity.
        rotation.yaw += angular_velocity.yaw * dt;
        // Build quaternion from yaw and apply to transform.
        transform.rotation = Quat::from_rotation_y(rotation.yaw);

        let is_small = matches!(actor_size, Some(ActorSize::Small));

        // Compute tentative new position.
        let mut new_position = transform.translation + velocity.0 * dt;

        // Small actors do not collide with static geometry.
        if !is_small {
            // Resolve static AABB collisions.
            for (static_transform, static_collider) in static_query.iter() {
                new_position = collision::resolve_circle_vs_aabb(
                    new_position,
                    collider.radius,
                    static_transform.translation,
                    static_collider.half_extents,
                );
            }

            // Resolve inward-facing wall segment collisions.
            for (wall_transform, wall_collider) in wall_static_query.iter() {
                for seg in &wall_collider.segments {
                    let a = wall_transform.transform_point(Vec3::new(seg.a.x, 0.0, seg.a.y));
                    let b = wall_transform.transform_point(Vec3::new(seg.b.x, 0.0, seg.b.y));
                    let normal = wall_transform.rotation * Vec3::new(seg.normal.x, 0.0, seg.normal.y);

                    new_position = collision::resolve_circle_vs_wall_segment(
                        new_position,
                        collider.radius,
                        a,
                        b,
                        normal,
                    );
                }
            }
        }

        resolved_positions.push((entity, new_position, collider.radius, is_small));
    }

    // Phase 2: resolve dynamic-dynamic collisions on the collected positions.
    for i in 0..resolved_positions.len() {
        for j in (i + 1)..resolved_positions.len() {
            let (_, pos_a, radius_a, small_a) = resolved_positions[i];
            let (_, pos_b, radius_b, small_b) = resolved_positions[j];

            if small_a || small_b {
                continue;
            }

            let (new_a, new_b) = collision::resolve_circle_pair(pos_a, radius_a, pos_b, radius_b);

            resolved_positions[i].1 = new_a;
            resolved_positions[j].1 = new_b;
        }
    }

    // Phase 3: write final positions back to transforms.
    let positions: HashMap<Entity, Vec3> = resolved_positions
        .into_iter()
        .map(|(entity, pos, _, _)| (entity, pos))
        .collect();

    for (entity, mut transform, ..) in dynamic_query.iter_mut() {
        if let Some(pos) = positions.get(&entity) {
            transform.translation = *pos;
        }
    }
}

/// Makes the world camera follow the currently focused entity.
pub fn camera_follow_system(
    focused_entity: Res<FocusedEntity>,
    view: Res<DiegeticView>,
    target_query: Query<(&Transform, Option<&FirstPersonCamera>), Without<ActiveWorldCamera>>,
    mut camera_query: Query<
        (&mut Transform, &mut Projection),
        (With<Camera>, With<ActiveWorldCamera>, Without<Player>),
    >,
) {
    if *view != DiegeticView::FirstPerson {
        return;
    }

    if let Some(entity) = focused_entity.entity {
        if let Ok((target_transform, first_person_opt)) = target_query.get(entity) {
            for (mut cam_transform, mut projection) in camera_query.iter_mut() {
                if let Some(first_person) = first_person_opt {
                    let eye_world = target_transform.translation
                        + target_transform.rotation * first_person.eye_offset;
                    cam_transform.translation = eye_world;
                    cam_transform.rotation = target_transform.rotation;
                    if let Projection::Perspective(persp) = &mut *projection {
                        persp.fov = first_person.fov;
                    }
                }
            }
        }
    }
}

/// Shows the active world camera only while the active view is `FirstPerson`.
pub fn world_camera_visibility_system(
    view: Res<DiegeticView>,
    mut camera_query: Query<&mut Visibility, With<ActiveWorldCamera>>,
) {
    for mut visibility in camera_query.iter_mut() {
        *visibility = if *view == DiegeticView::FirstPerson {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Handles diegetic view input.
pub fn schematic_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<DiegeticView>,
    mut layer: ResMut<SchematicLayer>,
    mut active_layers: ResMut<SchematicLayers>,
) {
    if keyboard.just_pressed(KeyCode::Slash) {
        *view = DiegeticView::CommandLine;
        info!("View switched to Command Line");
    }

    if keyboard.just_pressed(KeyCode::Backquote) {
        *view = match *view {
            DiegeticView::CommandLine => DiegeticView::Schematic,
            DiegeticView::Schematic => DiegeticView::CommandLine,
            DiegeticView::FirstPerson => DiegeticView::Schematic,
            DiegeticView::ConfigView => DiegeticView::Schematic,
        };
        info!("View toggled to {:?}", *view);
    }

    // Function-key layer toggling: open schematic and toggle layer.
    for (key, layer_enum) in [
        (KeyCode::F1, SchematicLayer::Structural),
        (KeyCode::F2, SchematicLayer::Electrical),
        (KeyCode::F3, SchematicLayer::Fluidics),
        (KeyCode::F4, SchematicLayer::Atmospherics),
        (KeyCode::F5, SchematicLayer::Thermal),
        (KeyCode::F6, SchematicLayer::Computer),
        (KeyCode::F7, SchematicLayer::Research),
        (KeyCode::F8, SchematicLayer::Docking),
        (KeyCode::F9, SchematicLayer::Sensors),
        (KeyCode::F10, SchematicLayer::Reserved),
        (KeyCode::F11, SchematicLayer::Reserved),
        (KeyCode::F12, SchematicLayer::Reserved),
    ] {
        if keyboard.just_pressed(key) {
            *view = DiegeticView::Schematic;
            toggle_layer(&mut active_layers, layer_enum);
            *layer = layer_enum;
            info!("Toggled schematic layer {:?}", layer_enum);
        }
    }

    let ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    for (key, layer_enum) in [
        (KeyCode::Digit1, SchematicLayer::Structural),
        (KeyCode::Digit2, SchematicLayer::Electrical),
        (KeyCode::Digit3, SchematicLayer::Fluidics),
        (KeyCode::Digit4, SchematicLayer::Atmospherics),
        (KeyCode::Digit5, SchematicLayer::Thermal),
        (KeyCode::Digit6, SchematicLayer::Computer),
        (KeyCode::Digit7, SchematicLayer::Research),
        (KeyCode::Digit8, SchematicLayer::Docking),
        (KeyCode::Digit9, SchematicLayer::Sensors),
    ] {
        if ctrl_pressed && keyboard.just_pressed(key) {
            *view = DiegeticView::Schematic;
            toggle_layer(&mut active_layers, layer_enum);
            *layer = layer_enum;
            info!("Toggled schematic layer {:?} via Ctrl+digit fallback", layer_enum);
        }
    }

    if ctrl_pressed && keyboard.just_pressed(KeyCode::Digit0) {
        *view = DiegeticView::Schematic;
        toggle_layer(&mut active_layers, SchematicLayer::Reserved);
        *layer = SchematicLayer::Reserved;
        info!("Toggled Reserved layer via Ctrl+0 fallback");
    }
}

fn toggle_layer(active_layers: &mut SchematicLayers, layer: SchematicLayer) {
    if !active_layers.active.remove(&layer) {
        active_layers.active.insert(layer);
    }
}

/// Handles number key focus switching.
pub fn drone_focus_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    terminal_input: Res<TerminalInputBuffer>,
    mut focused_entity: ResMut<FocusedEntity>,
    mut view: ResMut<DiegeticView>,
    drone_query: Query<(Entity, &Drone)>,
    _blackbox_query: Query<Entity, With<Blackbox>>,
) {
    if !terminal_input.current.is_empty() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Digit0) {
        *view = DiegeticView::ConfigView;
        info!("Opened fleet configuration view");
        return;
    }

    let key_map = [
        (KeyCode::Digit1, 1u8),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ];

    for (key, index) in key_map {
        if keyboard.just_pressed(key) {
            for (entity, drone) in drone_query.iter() {
                if drone.index == index {
                    focused_entity.entity = Some(entity);
                    *view = DiegeticView::FirstPerson;
                    info!("Focused entity set to drone {} and view switched to FirstPerson", index);
                    break;
                }
            }
            break;
        }
    }
}

/// Pauses the game when `Escape` is pressed.
pub fn pause_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut view: ResMut<DiegeticView>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if *view != DiegeticView::FirstPerson {
            *view = DiegeticView::FirstPerson;
            info!("Closed overlay and returned to FirstPerson view");
        } else {
            next_state.set(GameState::Paused);
        }
    }
}

/// Debug helper: press `K` to enter the kill screen.
pub fn debug_game_over_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyK) {
        next_state.set(GameState::GameOver);
    }
}

/// Run condition: true only when the game is in Playing state,
/// editor mode is inactive, and spectator mode is inactive.
pub fn playing_and_not_editor(
    state: Res<State<GameState>>,
    editor_mode: Res<EditorMode>,
    spectator_mode: Res<SpectatorMode>,
) -> bool {
    *state.get() == GameState::Playing && !editor_mode.active && !spectator_mode.active
}

/// Despawns all entities marked with `GameplayEntity` when leaving `Playing`.
pub fn despawn_gameplay_world(
    mut commands: Commands,
    query: Query<Entity, With<GameplayEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
