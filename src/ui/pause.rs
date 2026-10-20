use bevy::app::AppExit;
use bevy::prelude::*;

use crate::resources::{GameState, EditorMode, SpectatorMode};

#[derive(Component)]
pub struct PauseScreen;

#[derive(Component)]
pub enum PauseAction {
    Resume,
    MainMenu,
    EnterEditor,
    EnterSpectator,
    Quit,
}

pub fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseScreen,
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
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Paused",
                    TextStyle {
                        font: default(),
                        font_size: 70.0,
                        color: Color::WHITE,
                    },
                ),
            ));

            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            });

            parent
                .spawn((
                    PauseAction::Resume,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::rgb(0.2, 0.5, 0.2)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Resume",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            parent
                .spawn((
                    PauseAction::EnterEditor,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::rgb(0.2, 0.4, 0.6)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Enter Editor",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            parent
                .spawn((
                    PauseAction::EnterSpectator,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::rgb(0.4, 0.4, 0.6)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Enter Spectator",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            parent
                .spawn((
                    PauseAction::MainMenu,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::rgb(0.4, 0.4, 0.2)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Main Menu",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            parent
                .spawn((
                    PauseAction::Quit,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(10.0)),
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
                        "Quit to Desktop",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });
        });
}

pub fn despawn_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<PauseScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn pause_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &PauseAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut editor_mode: ResMut<EditorMode>,
    mut spectator_mode: ResMut<SpectatorMode>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                PauseAction::Resume => {
                    next_state.set(GameState::Playing);
                }
                PauseAction::EnterEditor => {
                    spectator_mode.active = false;
                    editor_mode.active = true;
                    next_state.set(GameState::Playing);
                }
                PauseAction::EnterSpectator => {
                    editor_mode.active = false;
                    spectator_mode.active = true;
                    next_state.set(GameState::Playing);
                }
                PauseAction::MainMenu => {
                    editor_mode.active = false;
                    spectator_mode.active = false;
                    next_state.set(GameState::MainMenu);
                }
                PauseAction::Quit => {
                    app_exit_events.send(AppExit);
                }
            }
        }
    }
}

pub fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
    }
}
