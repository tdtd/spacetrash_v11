use bevy::prelude::*;

use crate::resources::*;

/// Marker component for the root editor UI entity.
#[derive(Component)]
pub struct EditorScreen;

/// Components used for updating editor status text dynamically.
#[derive(Component)]
pub struct EditorViewText;

#[derive(Component)]
pub struct EditorLayerText;

/// Marker for the button that exits editor mode.
#[derive(Component)]
pub struct EditorExitButton;

/// Actions that can be triggered from the editor toolbar.
#[derive(Component)]
pub enum EditorToolAction {
    Select,
    Room,
    Door,
    Delete,
    Move,
}

/// Spawns the editor UI overlay.
fn spawn_editor_ui(commands: &mut Commands) {
    commands
        .spawn((
            EditorScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // Top toolbar
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.0),
                        left: Val::Px(10.0),
                        width: Val::Px(600.0),
                        height: Val::Px(50.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
                    ..default()
                })
                .with_children(|toolbar| {
                    for (label, action) in [
                        ("Select", EditorToolAction::Select),
                        ("Room", EditorToolAction::Room),
                        ("Door", EditorToolAction::Door),
                        ("Delete", EditorToolAction::Delete),
                        ("Move", EditorToolAction::Move),
                    ] {
                        toolbar
                            .spawn((
                                action,
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(90.0),
                                        height: Val::Px(40.0),
                                        margin: UiRect::all(Val::Px(4.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    background_color: BackgroundColor(Color::rgb(0.3, 0.3, 0.3)),
                                    ..default()
                                },
                            ))
                            .with_children(|button| {
                                button.spawn(TextBundle::from_section(
                                    label,
                                    TextStyle {
                                        font: default(),
                                        font_size: 20.0,
                                        color: Color::WHITE,
                                    },
                                ));
                            });
                    }

                    // Exit Editor button
                    toolbar
                        .spawn((
                            EditorExitButton,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(100.0),
                                    height: Val::Px(40.0),
                                    margin: UiRect::all(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::rgb(0.5, 0.2, 0.2)),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section(
                                "Exit Editor",
                                TextStyle {
                                    font: default(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });

            // Bottom status bar
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(10.0),
                        left: Val::Px(10.0),
                        width: Val::Px(600.0),
                        height: Val::Px(40.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn(TextBundle::from_section(
                        "Editor Mode",
                        TextStyle {
                            font: default(),
                            font_size: 20.0,
                            color: Color::GREEN,
                        },
                    ));

                    bar.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: default(),
                                font_size: 18.0,
                                color: Color::WHITE,
                            },
                        ),
                        EditorViewText,
                    ));

                    bar.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: default(),
                                font_size: 18.0,
                                color: Color::YELLOW,
                            },
                        ),
                        EditorLayerText,
                    ));
                });
        });
}

/// Monitors `EditorMode` and spawns/despawns the editor UI overlay as needed.
pub fn editor_ui_visibility_system(
    mut commands: Commands,
    editor_mode: Res<EditorMode>,
    existing: Query<Entity, With<EditorScreen>>,
) {
    if editor_mode.active {
        if existing.is_empty() {
            spawn_editor_ui(&mut commands);
        }
    } else {
        for entity in existing.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Updates the text in the editor bottom bar to show the current diegetic view and schematic layer.
pub fn update_editor_ui_text_system(
    view: Res<DiegeticView>,
    layer: Res<SchematicLayer>,
    mut view_query: Query<&mut Text, (With<EditorViewText>, Without<EditorLayerText>)>,
    mut layer_query: Query<&mut Text, (With<EditorLayerText>, Without<EditorViewText>)>,
) {
    if let Ok(mut text) = view_query.get_single_mut() {
        text.sections[0].value = format!(
            "View: {}",
            match *view {
                DiegeticView::FirstPerson => "First Person",
                DiegeticView::CommandLine => "Command Line",
                DiegeticView::Schematic => "Schematic Map",
                DiegeticView::ConfigView => "Config View",
            }
        );
    }
    if let Ok(mut text) = layer_query.get_single_mut() {
        text.sections[0].value = format!(
            "Layer: {}",
            match *layer {
                SchematicLayer::Structural => "Structural",
                SchematicLayer::Electrical => "Electrical",
                SchematicLayer::Fluidics => "Fluidics",
                SchematicLayer::Atmospherics => "Atmospherics",
                SchematicLayer::Thermal => "Thermal",
                SchematicLayer::Computer => "Computer/Network",
                SchematicLayer::Research => "Research/Tech",
                SchematicLayer::Docking => "Docking/Travel",
                SchematicLayer::Sensors => "Sensors/Tactical",
                SchematicLayer::Reserved => "Reserved",
            }
        );
    }
}

/// Handles clicks on the editor toolbar buttons and updates the active editor tool.
pub fn editor_tool_button_system(
    mut interaction_query: Query<
        (&Interaction, &EditorToolAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_tool: ResMut<EditorTool>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            *editor_tool = match action {
                EditorToolAction::Select => EditorTool::Select,
                EditorToolAction::Room => EditorTool::Room,
                EditorToolAction::Door => EditorTool::Door,
                EditorToolAction::Delete => EditorTool::Delete,
                EditorToolAction::Move => EditorTool::Move,
            };
        }
    }
}

/// Handles clicks on the Exit Editor button.
pub fn editor_exit_button_system(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<EditorExitButton>),
    >,
    mut editor_mode: ResMut<EditorMode>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            editor_mode.active = false;
        }
    }
}
