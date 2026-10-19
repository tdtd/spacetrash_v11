use bevy::prelude::*;

use crate::resources::GameState;

#[derive(Component)]
pub struct LoadingScreen;

#[derive(Resource)]
pub struct LoadingTimer(pub Timer);

pub fn spawn_loading_screen(mut commands: Commands) {
    commands.insert_resource(LoadingTimer(Timer::from_seconds(1.0, TimerMode::Once)));

    commands
        .spawn((
            LoadingScreen,
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
                background_color: BackgroundColor(Color::BLACK),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Loading...",
                    TextStyle {
                        font: default(),
                        font_size: 60.0,
                        color: Color::WHITE,
                    },
                ),
            ));
        });
}

pub fn despawn_loading_screen(
    mut commands: Commands,
    query: Query<Entity, With<LoadingScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn loading_screen_tick(
    time: Res<Time>,
    mut timer: ResMut<LoadingTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        next_state.set(GameState::MainMenu);
    }
}
