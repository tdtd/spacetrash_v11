use bevy::app::AppExit;
use bevy::prelude::*;

use crate::resources::scenario::{ActiveScenario, ScenarioCatalog};
use crate::resources::{EditorMode, GameState};

#[derive(Component)]
pub struct MainMenuScreen;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainMenuLayer {
    #[default]
    Main,
    Play,
    NewGameScenarioSelect,
    Settings,
}

#[derive(Component)]
pub struct MainMenuLayerContainer {
    pub layer: MainMenuLayer,
}

#[derive(Component)]
pub enum MainMenuAction {
    LoadPlay,
    NewGame,
    LoadGame,
    OpenSettings,
    Back,
    SelectScenario(String),
    Quit,
}

pub fn spawn_main_menu(
    mut commands: Commands,
    scenario_catalog: Res<ScenarioCatalog>,
) {
    commands.insert_resource(MainMenuLayer::default());

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
            // Layer 1: Main
            parent
                .spawn((
                    MainMenuLayerContainer {
                        layer: MainMenuLayer::Main,
                    },
                    NodeBundle {
                        style: menu_layer_style(),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Spacetrash v11",
                        TextStyle {
                            font: default(),
                            font_size: 80.0,
                            color: Color::WHITE,
                        },
                    ));

                    p.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    });

                    menu_button(p, MainMenuAction::LoadPlay, "Load / Play", 240.0, 50.0);
                    menu_button(p, MainMenuAction::OpenSettings, "Settings", 240.0, 50.0);
                    menu_button(p, MainMenuAction::Quit, "Quit", 240.0, 50.0);
                });

            // Layer 2: Load/Play
            parent
                .spawn((
                    MainMenuLayerContainer {
                        layer: MainMenuLayer::Play,
                    },
                    NodeBundle {
                        style: menu_layer_style(),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Load / Play",
                        TextStyle {
                            font: default(),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    ));

                    p.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    });

                    menu_button(p, MainMenuAction::NewGame, "New Game", 240.0, 50.0);
                    menu_button(p, MainMenuAction::LoadGame, "Load Game", 240.0, 50.0);
                    menu_button(p, MainMenuAction::Back, "Back", 240.0, 50.0);
                });

            // Layer 3: New Game Scenario Select
            parent
                .spawn((
                    MainMenuLayerContainer {
                        layer: MainMenuLayer::NewGameScenarioSelect,
                    },
                    NodeBundle {
                        style: menu_layer_style(),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Choose Scenario",
                        TextStyle {
                            font: default(),
                            font_size: 56.0,
                            color: Color::WHITE,
                        },
                    ));

                    p.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    });

                    for (id, scenario) in &scenario_catalog.scenarios {
                        menu_button(
                            p,
                            MainMenuAction::SelectScenario(id.clone()),
                            &scenario.name,
                            240.0,
                            40.0,
                        );
                    }

                    menu_button(p, MainMenuAction::Back, "Back", 240.0, 50.0);
                });

            // Layer 4: Settings placeholder
            parent
                .spawn((
                    MainMenuLayerContainer {
                        layer: MainMenuLayer::Settings,
                    },
                    NodeBundle {
                        style: menu_layer_style(),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Settings",
                        TextStyle {
                            font: default(),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    ));

                    p.spawn(TextBundle::from_section(
                        "Settings are not implemented yet.",
                        TextStyle {
                            font: default(),
                            font_size: 24.0,
                            color: Color::rgb(0.7, 0.7, 0.7),
                        },
                    ));

                    p.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    });

                    menu_button(p, MainMenuAction::Back, "Back", 240.0, 50.0);
                });
        });
}

fn menu_layer_style() -> Style {
    Style {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn menu_button(parent: &mut ChildBuilder, action: MainMenuAction, label: &str, width: f32, height: f32) {
    parent
        .spawn((
            action,
            ButtonBundle {
                style: Style {
                    width: Val::Px(width),
                    height: Val::Px(height),
                    margin: UiRect::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.25, 0.25, 0.3)),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                label.to_string(),
                TextStyle {
                    font: default(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ));
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

pub fn main_menu_visibility_system(
    layer: Res<MainMenuLayer>,
    mut query: Query<(&MainMenuLayerContainer, &mut Visibility)>,
) {
    for (container, mut visibility) in query.iter_mut() {
        *visibility = if container.layer == *layer {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn main_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &MainMenuAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut editor_mode: ResMut<EditorMode>,
    mut active_scenario: ResMut<ActiveScenario>,
    mut layer: ResMut<MainMenuLayer>,
    scenario_catalog: Res<ScenarioCatalog>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            MainMenuAction::LoadPlay => {
                *layer = MainMenuLayer::Play;
            }
            MainMenuAction::NewGame => {
                *layer = MainMenuLayer::NewGameScenarioSelect;
            }
            MainMenuAction::LoadGame => {
                *layer = MainMenuLayer::Main;
            }
            MainMenuAction::OpenSettings => {
                *layer = MainMenuLayer::Settings;
            }
            MainMenuAction::Back => {
                *layer = MainMenuLayer::Main;
            }
            MainMenuAction::SelectScenario(id) => {
                editor_mode.active = false;
                if let Some(scenario) = scenario_catalog.scenarios.get(id.as_str()) {
                    active_scenario.scenario = Some(scenario.clone());
                }
                *layer = MainMenuLayer::Main;
                next_state.set(GameState::Playing);
            }
            MainMenuAction::Quit => {
                app_exit_events.send(AppExit);
            }
        }
    }
}

pub fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut layer: ResMut<MainMenuLayer>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match *layer {
            MainMenuLayer::Main => {
                app_exit_events.send(AppExit);
            }
            _ => {
                *layer = MainMenuLayer::Main;
            }
        }
    }
}
