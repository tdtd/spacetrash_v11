use bevy::app::AppExit;
use bevy::prelude::*;

use crate::resources::GameState;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub enum GameOverAction {
    Restart,
    MainMenu,
    Quit,
}

pub fn spawn_game_over_screen(mut commands: Commands) {
    commands
        .spawn((
            GameOverScreen,
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
                background_color: BackgroundColor(Color::rgba(0.6, 0.1, 0.1, 0.9)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "You died.",
                    TextStyle {
                        font: default(),
                        font_size: 90.0,
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
                    GameOverAction::Restart,
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
                        "Restart",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            parent
                .spawn((
                    GameOverAction::MainMenu,
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
                    GameOverAction::Quit,
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

pub fn despawn_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameOverScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn game_over_button_system(
    mut interaction_query: Query<
        (&Interaction, &GameOverAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                GameOverAction::Restart => {
                    next_state.set(GameState::Playing);
                }
                GameOverAction::MainMenu => {
                    next_state.set(GameState::MainMenu);
                }
                GameOverAction::Quit => {
                    app_exit_events.send(AppExit);
                }
            }
        }
    }
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}
