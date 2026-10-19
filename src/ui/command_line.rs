use bevy::prelude::*;

use crate::components::{Blackbox, Drone};
use crate::resources::{DiegeticView, FocusedEntity};

#[derive(Component)]
pub struct CommandLineScreen;

#[derive(Component)]
pub struct CommandLineFocusText;

pub fn spawn_command_line(commands: &mut Commands) {
    commands
        .spawn((
            CommandLineScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexEnd,
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 1.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "COMMAND LINE",
                TextStyle {
                    font: default(),
                    font_size: 32.0,
                    color: Color::rgb(0.2, 1.0, 0.2),
                },
            ));

            parent.spawn((
                TextBundle::from_section(
                    "No focus",
                    TextStyle {
                        font: default(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ),
                CommandLineFocusText,
            ));

            parent.spawn(TextBundle::from_section(
                "> _",
                TextStyle {
                    font: default(),
                    font_size: 26.0,
                    color: Color::rgb(0.8, 1.0, 0.8),
                },
            ));
        });
}

pub fn despawn_command_line(
    commands: &mut Commands,
    query: &Query<Entity, With<CommandLineScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn command_line_visibility_system(
    mut commands: Commands,
    view: Res<DiegeticView>,
    existing: Query<Entity, With<CommandLineScreen>>,
) {
    let should_show = *view == DiegeticView::CommandLine;

    if should_show {
        if existing.is_empty() {
            spawn_command_line(&mut commands);
        }
    } else {
        despawn_command_line(&mut commands, &existing);
    }
}

pub fn update_command_line_focus_text(
    focused_entity: Res<FocusedEntity>,
    blackbox_query: Query<Entity, With<Blackbox>>,
    drone_query: Query<(Entity, &Drone)>,
    mut text_query: Query<&mut Text, With<CommandLineFocusText>>,
) {
    let Some(entity) = focused_entity.entity else {
        return;
    };

    let focus_name = if blackbox_query.get(entity).is_ok() {
        "Blackbox".to_string()
    } else if let Ok((_, drone)) = drone_query.get(entity) {
        format!("Drone {}", drone.index)
    } else {
        "Unknown".to_string()
    };

    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!("Focused: {}", focus_name);
    }
}
