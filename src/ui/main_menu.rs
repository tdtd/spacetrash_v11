use bevy::app::AppExit;
use bevy::prelude::*;

use crate::resources::GameState;

#[derive(Component)]
pub struct MainMenuScreen;

#[derive(Component)]
pub enum MainMenuAction {
    StartGame,
    Quit,
}

pub fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((
            MainMenuScreen,
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
                background_color: BackgroundColor(Color::rgb(0.05, 0.05, 0.1)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Spacetrash v11",
                    TextStyle {
                        font: default(),
                        font_size: 80.0,
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
                    MainMenuAction::StartGame,
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
                        "Start Game",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            parent
                .spawn((
                    MainMenuAction::Quit,
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
                        "Quit",
                        TextStyle {
                            font: default(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                });
        });
}

pub fn despawn_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn main_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &MainMenuAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                MainMenuAction::StartGame => {
                    next_state.set(GameState::Playing);
                }
                MainMenuAction::Quit => {
                    app_exit_events.send(AppExit);
                }
            }
        }
    }
}

pub fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
