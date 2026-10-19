use bevy::prelude::*;
use bevy::render::camera::ClearColorConfig;
use bevy::ui::IsDefaultUiCamera;
use bevy_prototype_lyon::prelude::ShapePlugin;

use std::collections::HashMap;

use crate::components::ActiveWorldCamera;
use crate::resources::*;
use crate::resources::scenario::{default_ship_template, ShipTemplateLibrary};
use crate::systems::*;
use crate::ui::loading::*;
use crate::ui::main_menu::*;
use crate::ui::pause::*;
use crate::ui::game_over::*;
use crate::ui::hud::*;
use crate::ui::editor::*;
use crate::ui::command_line::*;
use crate::ui::schematic::*;
use crate::ui::config_view::*;

fn setup_cameras(mut commands: Commands) {
    // 3D camera for the world, marked as the active world camera.
    commands.spawn((
        ActiveWorldCamera,
        Camera3dBundle {
            camera: Camera {
                order: 0,
                clear_color: ClearColorConfig::Default,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));

    // 2D camera that is responsible for rendering UI.
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 10,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        IsDefaultUiCamera,
    ));
}

fn debug_ui_on_main_menu(
    ui_nodes: Query<(), With<Node>>,
    ui_cameras: Query<(), (With<Camera>, With<IsDefaultUiCamera>)>,
    main_menu_screens: Query<(), With<MainMenuScreen>>,
    state: Res<State<GameState>>,
) {
    info!(
        "State: {:?}, UI Nodes: {}, Default UI Cameras: {}, MainMenuScreens: {}",
        state.get(),
        ui_nodes.iter().count(),
        ui_cameras.iter().count(),
        main_menu_screens.iter().count(),
    );
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin)
        .init_state::<GameState>()
        .insert_resource(PhysicsConfig::default())
        .insert_resource(InputConfig::default())
        .insert_resource(EditorMode::default())
        .insert_resource(EditorTool::default())
        .insert_resource(DiegeticView::default())
        .insert_resource(SchematicLayer::default())
        .insert_resource(SchematicLayers::default())
        .insert_resource(FocusedEntity::default())
        .insert_resource(ShipTemplateLibrary {
            templates: {
                let mut map = HashMap::new();
                map.insert("test_ship".to_string(), default_ship_template());
                map
            },
        })

        // Persistent cameras for world and UI.
        .add_systems(Startup, setup_cameras)

        // Loading screen
        .add_systems(OnEnter(GameState::Loading), spawn_loading_screen)
        .add_systems(OnExit(GameState::Loading), despawn_loading_screen)
        .add_systems(Update, loading_screen_tick.run_if(in_state(GameState::Loading)))

        // Main menu
        .add_systems(
            OnEnter(GameState::MainMenu),
            (spawn_main_menu, debug_ui_on_main_menu).chain(),
        )
        .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
        .add_systems(
            Update,
            (main_menu_input, main_menu_button_system).run_if(in_state(GameState::MainMenu)),
        )

        // Playing
        .add_systems(OnEnter(GameState::Playing), setup_scene)
        .add_systems(OnEnter(GameState::Playing), spawn_hud)
        .add_systems(OnExit(GameState::Playing), despawn_gameplay_world)
        .add_systems(OnExit(GameState::Playing), despawn_hud)
        .add_systems(
            Update,
            (
                player_input_system,
                movement_system,
                camera_follow_system,
                world_camera_visibility_system,
                schematic_input_system,
                pause_input_system,
                debug_game_over_system,
            )
                .run_if(playing_and_not_editor),
        )
        .add_systems(
            Update,
            (
                editor_ui_visibility_system,
                editor_tool_button_system,
                update_editor_ui_text_system,
                editor_exit_button_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Drone focus switching works in both normal play and editor mode.
        .add_systems(Update, drone_focus_input_system.run_if(in_state(GameState::Playing)))
        // Persistent schematic status text in the HUD.
        .add_systems(Update, update_schematic_hud.run_if(in_state(GameState::Playing)))
        // Diegetic overlays: command line, schematic map, and config view.
        .add_systems(
            Update,
            (
                command_line_visibility_system,
                schematic_visibility_system,
                config_view_visibility_system,
                update_command_line_focus_text,
                update_schematic_legend,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Update the schematic map every frame while the schematic is open.
        .add_systems(
            Update,
            (update_ascii_schematic_map)
                .after(schematic_visibility_system)
                .run_if(in_state(GameState::Playing)),
        )

        // Pause
        .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
        .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
        .add_systems(
            Update,
            (pause_menu_input, pause_menu_button_system).run_if(in_state(GameState::Paused)),
        )

        // Game over
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), despawn_game_over_screen)
        .add_systems(Update, game_over_input.run_if(in_state(GameState::GameOver)))

        .run();
}
