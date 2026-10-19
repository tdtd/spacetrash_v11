use bevy::prelude::*;

use crate::resources::{DiegeticView, SchematicLayer};

#[derive(Component)]
pub struct HudScreen;

#[derive(Component)]
pub struct SchematicViewText;

#[derive(Component)]
pub struct SchematicLayerText;

pub fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Health: 100  Oxygen: 100",
                    TextStyle {
                        font: default(),
                        font_size: 28.0,
                        color: Color::WHITE,
                    },
                ),
            ));

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: default(),
                        font_size: 18.0,
                        color: Color::rgb(0.8, 0.8, 1.0),
                    },
                ),
                SchematicViewText,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: default(),
                        font_size: 18.0,
                        color: Color::YELLOW,
                    },
                ),
                SchematicLayerText,
            ));
        });
}

pub fn update_schematic_hud(
    view: Res<DiegeticView>,
    layer: Res<SchematicLayer>,
    mut view_query: Query<&mut Text, (With<SchematicViewText>, Without<SchematicLayerText>)>,
    mut layer_query: Query<&mut Text, (With<SchematicLayerText>, Without<SchematicViewText>)>,
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

pub fn despawn_hud(
    mut commands: Commands,
    query: Query<Entity, With<HudScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
