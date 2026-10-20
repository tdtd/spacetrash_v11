use bevy::prelude::*;

use crate::resources::{SpectatorMode, SpectatorReplay};

#[derive(Component)]
pub struct SpectatorScreen;

#[derive(Component)]
pub struct SpectatorExitButton;

#[derive(Component)]
pub struct SpectatorTickText;

#[derive(Component)]
pub enum SpectatorAction {
    PlayPause,
    Step,
    SpeedUp,
    SpeedDown,
}

pub fn spawn_spectator_ui(commands: &mut Commands) {
    commands
        .spawn((
            SpectatorScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Percent(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.6)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "SPECTATOR MODE",
                TextStyle {
                    font: default(),
                    font_size: 60.0,
                    color: Color::rgb(0.4, 0.8, 1.0),
                },
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Tick: 0 / 0",
                    TextStyle {
                        font: default(),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ),
                SpectatorTickText,
            ));

            parent.spawn(TextBundle::from_section(
                "Press ESC to exit spectator mode",
                TextStyle {
                    font: default(),
                    font_size: 20.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                },
            ));

            // Control row
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Px(16.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|controls| {
                    // Play/Pause button
                    controls
                        .spawn((
                            SpectatorAction::PlayPause,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(100.0),
                                    height: Val::Px(40.0),
                                    margin: UiRect::all(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::rgb(0.2, 0.3, 0.6)),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section(
                                "Play/Pause",
                                TextStyle {
                                    font: default(),
                                    font_size: 18.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });

                    // Step button
                    controls
                        .spawn((
                            SpectatorAction::Step,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(60.0),
                                    height: Val::Px(40.0),
                                    margin: UiRect::all(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::rgb(0.2, 0.3, 0.6)),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section(
                                "Step",
                                TextStyle {
                                    font: default(),
                                    font_size: 18.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });

                    // Speed up
                    controls
                        .spawn((
                            SpectatorAction::SpeedUp,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(50.0),
                                    height: Val::Px(40.0),
                                    margin: UiRect::all(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::rgb(0.2, 0.3, 0.6)),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section(
                                "+",
                                TextStyle {
                                    font: default(),
                                    font_size: 18.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });

                    // Speed down
                    controls
                        .spawn((
                            SpectatorAction::SpeedDown,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(50.0),
                                    height: Val::Px(40.0),
                                    margin: UiRect::all(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::rgb(0.2, 0.3, 0.6)),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section(
                                "-",
                                TextStyle {
                                    font: default(),
                                    font_size: 18.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });

            // Exit button
            parent
                .spawn((
                    SpectatorExitButton,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(220.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::rgb(0.5, 0.2, 0.2)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Exit Spectator",
                        TextStyle {
                            font: default(),
                            font_size: 28.0,
                            color: Color::WHITE,
                        },
                    ));
                });
        });
}

pub fn despawn_spectator_ui(
    commands: &mut Commands,
    query: &Query<Entity, With<SpectatorScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spectator_visibility_system(
    mut commands: Commands,
    spectator_mode: Res<SpectatorMode>,
    existing: Query<Entity, With<SpectatorScreen>>,
) {
    if spectator_mode.active {
        if existing.is_empty() {
            spawn_spectator_ui(&mut commands);
        }
    } else {
        despawn_spectator_ui(&mut commands, &existing);
    }
}

pub fn spectator_exit_button_system(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<SpectatorExitButton>),
    >,
    mut spectator_mode: ResMut<SpectatorMode>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            spectator_mode.active = false;
        }
    }
}

pub fn spectator_action_button_system(
    mut interaction_query: Query<
        (&Interaction, &SpectatorAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut replay: ResMut<SpectatorReplay>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            SpectatorAction::PlayPause => {
                replay.is_playing = !replay.is_playing;
            }
            SpectatorAction::Step => {
                replay.is_playing = false;
                let total = replay.event_log.events.len() as u32;
                replay.current_tick = (replay.current_tick + 1).min(total.saturating_sub(1));
            }
            SpectatorAction::SpeedUp => {
                replay.playback_speed = (replay.playback_speed * 1.5).min(5.0);
            }
            SpectatorAction::SpeedDown => {
                replay.playback_speed = (replay.playback_speed * 0.5).max(0.1);
            }
        }
    }
}

pub fn spectator_update_ui_system(
    replay: Res<SpectatorReplay>,
    mut text_query: Query<&mut Text, With<SpectatorTickText>>,
) {
    let total = replay.event_log.events.len() as u32;
    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!(
            "Tick: {} / {}   Speed: {}x{}",
            replay.current_tick,
            total,
            if replay.is_playing { "" } else { "(paused) " },
            replay.playback_speed
        );
    }
}
