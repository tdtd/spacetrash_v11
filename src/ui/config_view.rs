use bevy::prelude::*;

use crate::components::{Blackbox, Controllable, Drone};
use crate::resources::DiegeticView;

/// Marker component for the root config view UI entity.
#[derive(Component)]
pub struct ConfigScreen;

/// Spawns the fleet configuration view.
pub fn spawn_config_view(
    commands: &mut Commands,
    blackbox_query: &Query<Entity, (With<Blackbox>, With<Controllable>)>,
    drone_query: &Query<&Drone, With<Controllable>>,
) {
    commands
        .spawn((
            ConfigScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.02, 0.05, 0.1, 1.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "FLEET CONFIGURATION",
                TextStyle {
                    font: default(),
                    font_size: 38.0,
                    color: Color::rgb(0.0, 1.0, 1.0),
                },
            ));

            // Display Blackbox info.
            if let Ok(_) = blackbox_query.get_single() {
                parent.spawn(TextBundle::from_section(
                    "Blackbox: Upgrades: [none]",
                    TextStyle {
                        font: default(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ));
            } else {
                parent.spawn(TextBundle::from_section(
                    "Blackbox: not found",
                    TextStyle {
                        font: default(),
                        font_size: 20.0,
                        color: Color::YELLOW,
                    },
                ));
            }

            // Display drone info.
            for drone in drone_query.iter() {
                parent.spawn(TextBundle::from_section(
                    format!("Drone {}: Upgrades: [none]", drone.index),
                    TextStyle {
                        font: default(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ));
            }
        });
}

/// Despawns the config view screen.
pub fn despawn_config_view(
    commands: &mut Commands,
    query: &Query<Entity, With<ConfigScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Toggles the config view based on active `DiegeticView`.
pub fn config_view_visibility_system(
    mut commands: Commands,
    view: Res<DiegeticView>,
    existing: Query<Entity, With<ConfigScreen>>,
    blackbox_query: Query<Entity, (With<Blackbox>, With<Controllable>)>,
    drone_query: Query<&Drone, With<Controllable>>,
) {
    let should_show = *view == DiegeticView::ConfigView;

    if should_show {
        if existing.is_empty() {
            spawn_config_view(&mut commands, &blackbox_query, &drone_query);
        }
    } else {
        despawn_config_view(&mut commands, &existing);
    }
}
