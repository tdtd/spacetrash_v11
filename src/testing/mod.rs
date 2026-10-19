use bevy::prelude::*;
use bevy::time::{TimePlugin, TimeUpdateStrategy};
use std::collections::HashMap;
use std::time::Duration;

use crate::components::{
    AabbCollider, ActorSize, AngularVelocity, CircleCollider, Rotation, Velocity,
};
use crate::resources::scenario::{default_ship_template, ShipTemplateLibrary};
use crate::resources::*;

pub mod debug_map;
pub mod map_assertions;
pub mod map_report;
pub mod wfc_trace;

/// Creates the smallest deterministic `App` suitable for future headless tests.
pub fn test_app() -> App {
    let mut app = App::new();

    // Insert all resources before adding plugins so the time strategy is
    // available when the TimePlugin's time_system runs.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        PhysicsConfig::default().fixed_timestep,
    )))
    .insert_resource(PhysicsConfig::default())
    .insert_resource(InputConfig::default())
    .insert_resource(EditorMode::default())
    .insert_resource(EditorTool::default())
    .insert_resource(DiegeticView::default())
    .insert_resource(SchematicLayer::default())
    .insert_resource(SchematicLayers::default())
    .insert_resource(FocusedEntity::default())
    .insert_resource(ButtonInput::<KeyCode>::default())
    .insert_resource(ShipTemplateLibrary {
        templates: {
            let mut map = HashMap::new();
            map.insert("test_ship".to_string(), default_ship_template());
            map
        },
    })
    .init_resource::<Time>();

    // Disable TimePlugin because we manage Time manually in run_n_ticks.
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>())
        .init_state::<GameState>();

    app
}

/// Spawns a dynamic actor with a circle collider and `Standard` actor size.
pub fn spawn_actor(app: &mut App, radius: f32) -> Entity {
    spawn_actor_with_size(app, radius, ActorSize::Standard)
}

/// Spawns a dynamic actor with a circle collider and the given actor size.
pub fn spawn_actor_with_size(app: &mut App, radius: f32, size: ActorSize) -> Entity {
    app.world
        .spawn((
            Transform::from_xyz(0.0, 0.25, 0.0),
            Velocity::default(),
            Rotation::default(),
            AngularVelocity::default(),
            CircleCollider { radius },
            size,
        ))
        .id()
}

/// Spawns a static wall with an AABB collider.
pub fn spawn_static_wall(app: &mut App, half_extents: Vec3) -> Entity {
    app.world
        .spawn((
            Transform::default(),
            AabbCollider { half_extents },
        ))
        .id()
}

/// Presses a key in the shared input resource.
pub fn press_key(app: &mut App, key: KeyCode) {
    let mut input = app.world.resource_mut::<ButtonInput<KeyCode>>();
    input.press(key);
}

/// Releases a key in the shared input resource.
pub fn release_key(app: &mut App, key: KeyCode) {
    let mut input = app.world.resource_mut::<ButtonInput<KeyCode>>();
    input.release(key);
}

/// Advances the app by `ticks`, ensuring each tick sees a fixed time delta.
pub fn run_n_ticks(app: &mut App, ticks: usize) {
    let dt = Duration::from_secs_f32(PhysicsConfig::default().fixed_timestep);
    for _ in 0..ticks {
        app.world.resource_mut::<Time>().advance_by(dt);
        app.update();
    }
}
