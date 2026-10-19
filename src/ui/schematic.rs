use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::ImageSampler;

use crate::components::{Blackbox, Controllable, Drone, GridCoordinates, TileNode};
use crate::procgen::scaffold::{EdgeKind, RoomGraph, RoomType, SystemFlags};
use crate::procgen::tile::TileType;
use crate::resources::{DiegeticView, SchematicLayer, SchematicLayers, CurrentRoomGraph};

#[derive(Component)]
pub struct SchematicScreen;

#[derive(Component)]
pub struct SchematicLegendText;

#[derive(Component)]
pub struct SchematicAsciiText;

#[derive(Component)]
pub struct SchematicRenderCanvas;

const SCHEMATIC_TILE_PIXELS: usize = 8;

pub fn spawn_schematic(commands: &mut Commands, mut images: ResMut<Assets<Image>>) {
    const LAYERS: [(SchematicLayer, &str); 12] = [
        (SchematicLayer::Structural, "Structural"),
        (SchematicLayer::Electrical, "Electrical"),
        (SchematicLayer::Fluidics, "Fluidics"),
        (SchematicLayer::Atmospherics, "Atmospherics"),
        (SchematicLayer::Thermal, "Thermal"),
        (SchematicLayer::Computer, "Computer/Network"),
        (SchematicLayer::Research, "Research/Tech"),
        (SchematicLayer::Docking, "Docking/Travel"),
        (SchematicLayer::Sensors, "Sensors/Tactical"),
        (SchematicLayer::Reserved, "Reserved"),
        (SchematicLayer::Reserved, "Reserved"),
        (SchematicLayer::Reserved, "Reserved"),
    ];

    let legend_value = format!(
        "F1-{} / Ctrl+1-{}: {}\nBacktick: command/schematic | //: command line | Esc: return to first person",
        9,
        9,
        LAYERS
            .iter()
            .enumerate()
            .filter_map(|(i, (_, name))| {
                if i < 9 {
                    Some(format!("F{}={}", i + 1, name))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    let canvas_handle = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0u8, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    ));

    commands
        .spawn((
            SchematicScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.02, 0.05, 0.1, 1.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "SCHEMATIC MAP",
                TextStyle {
                    font: default(),
                    font_size: 38.0,
                    color: Color::rgb(0.0, 1.0, 1.0),
                },
            ));

            parent.spawn((
                TextBundle::from_section(
                    "No layers active",
                    TextStyle {
                        font: default(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ),
                SchematicLegendText,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Structural map will appear here.",
                    TextStyle {
                        font: default(),
                        font_size: 18.0,
                        color: Color::rgb(0.6, 0.6, 0.6),
                    },
                ),
                SchematicAsciiText,
            ));

            parent.spawn(TextBundle::from_section(
                legend_value,
                TextStyle {
                    font: default(),
                    font_size: 16.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                },
            ));

            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(1.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    image: UiImage::new(canvas_handle),
                    ..default()
                },
                SchematicRenderCanvas,
            ));
        });
}

pub fn despawn_schematic(
    commands: &mut Commands,
    query: &Query<Entity, With<SchematicScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn schematic_visibility_system(
    mut commands: Commands,
    view: Res<DiegeticView>,
    existing: Query<Entity, With<SchematicScreen>>,
    images: ResMut<Assets<Image>>,
) {
    let should_show = *view == DiegeticView::Schematic;

    if should_show {
        if existing.is_empty() {
            spawn_schematic(&mut commands, images);
        }
    } else {
        despawn_schematic(&mut commands, &existing);
    }
}

pub fn update_ascii_schematic_map(
    _commands: Commands,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut text_query: Query<&mut Text, With<SchematicAsciiText>>,
    mut visibility_query: Query<
        (
            &mut Visibility,
            Option<&SchematicAsciiText>,
            Option<&SchematicRenderCanvas>,
        ),
        Or<(With<SchematicAsciiText>, With<SchematicRenderCanvas>)>,
    >,
    mut canvas_query: Query<(&mut UiImage, &mut Style), With<SchematicRenderCanvas>>,
    tiles: Query<(&GridCoordinates, &TileNode, &Transform)>,
    bots: Query<(&Transform, Option<&Blackbox>, Option<&Drone>), With<Controllable>>,
    layer: Res<SchematicLayer>,
    current_graph: Option<Res<CurrentRoomGraph>>,
) {
    let mut text = match text_query.get_single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    let tile_data: Vec<(&GridCoordinates, &TileNode, &Transform)> = tiles.iter().collect();

    if tile_data.is_empty() {
        text.sections[0].value = "No tiles found".to_string();
        return;
    }

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for (coords, _, _) in tile_data.iter() {
        min_x = min_x.min(coords.x);
        max_x = max_x.max(coords.x);
        min_y = min_y.min(coords.y);
        max_y = max_y.max(coords.y);
    }

    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;
    let pixel_width = width * SCHEMATIC_TILE_PIXELS;
    let pixel_height = height * SCHEMATIC_TILE_PIXELS;

    let is_structural = *layer == SchematicLayer::Structural;
    let is_terminal = *layer == SchematicLayer::Computer;

    // Update visibility of the ASCII text and the canvas image based on active layer.
    for (mut visibility, ascii_marker, canvas_marker) in visibility_query.iter_mut() {
        if ascii_marker.is_some() {
            *visibility = if is_structural || is_terminal {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        } else if canvas_marker.is_some() {
            *visibility = if is_structural || is_terminal {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }

    // Structural / terminal layers use the text entity.
    if is_structural {
        let mut grid = vec![vec![' '; width]; height];

        for (coords, tile_node, _) in tile_data.iter() {
            let gx = (coords.x - min_x) as usize;
            let gy = (max_y - coords.y) as usize;
            grid[gy][gx] = tile_node.tile_type.schematic_char();
        }

        // Overlay bots.
        for (bot_transform, blackbox_opt, drone_opt) in bots.iter() {
            let mut best_dist_sq = f32::MAX;
            let mut best_coord = (min_x, min_y);

            for (coords, _, tile_transform) in tile_data.iter() {
                let dx = bot_transform.translation.x - tile_transform.translation.x;
                let dz = bot_transform.translation.z - tile_transform.translation.z;
                let dist_sq = dx * dx + dz * dz;

                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    best_coord = (coords.x, coords.y);
                }
            }

            let grid_x = (best_coord.0 - min_x) as usize;
            let grid_y = (max_y - best_coord.1) as usize;

            let glyph = if blackbox_opt.is_some() {
                '@'
            } else if let Some(drone) = drone_opt {
                (b'0' + drone.index) as char
            } else {
                '?'
            };

            if grid_x < width && grid_y < height {
                grid[grid_y][grid_x] = glyph;
            }
        }

        let mut result = String::new();
        for row in grid {
            result.push_str(&row.iter().collect::<String>());
            result.push('\n');
        }

        text.sections[0].value = result;
        text.sections[0].style.color = Color::rgb(0.6, 0.6, 0.6);
        return;
    }

    if is_terminal {
        let terminal_text = if let Some(res) = current_graph.as_ref() {
            render_terminal_transcript(&res.graph)
        } else {
            "No room graph loaded.\n> _".to_string()
        };

        text.sections[0].value = terminal_text;
        text.sections[0].style.color = Color::rgb(0.2, 1.0, 0.2);
        return;
    }

    // Non-text layers are drawn into an image.
    if canvas_query.is_empty() {
        return;
    }

    let image = if let Some(res) = current_graph.as_ref() {
        render_schematic_image(
            *layer,
            &res.graph,
            pixel_width,
            pixel_height,
            min_x,
            max_y,
            &tile_data,
            time.elapsed_seconds(),
        )
    } else {
        render_placeholder_image(pixel_width, pixel_height)
    };

    let handle = images.add(image);
    for (mut ui_image, mut style) in canvas_query.iter_mut() {
        ui_image.texture = handle.clone();
        style.width = Val::Px(pixel_width as f32);
        style.height = Val::Px(pixel_height as f32);
    }
}

fn room_center(room: &crate::procgen::scaffold::RoomNode) -> (f32, f32) {
    (
        room.shape.x + room.shape.width * 0.5,
        room.shape.y + room.shape.height * 0.5,
    )
}

fn put_pixel(buf: &mut [u8], width: usize, height: usize, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let idx = ((y as usize * width + x as usize) * 4).min(buf.len().saturating_sub(4));
    buf[idx..idx + 4].copy_from_slice(&color);
}

fn draw_line(buf: &mut [u8], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
    let mut x0 = x0;
    let mut y0 = y0;
    let x1 = x1.max(0).min(width as i32 - 1);
    let y1 = y1.max(0).min(height as i32 - 1);

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_pixel(buf, width, height, x0, y0, color);

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_circle(buf: &mut [u8], width: usize, height: usize, cx: i32, cy: i32, radius: i32, color: [u8; 4]) {
    for y in -radius..=radius {
        for x in -radius..=radius {
            if x * x + y * y <= radius * radius {
                put_pixel(buf, width, height, cx + x, cy + y, color);
            }
        }
    }
}

fn fill_rect(buf: &mut [u8], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
    for y in y0.max(0)..=y1.min(height as i32 - 1) {
        for x in x0.max(0)..=x1.min(width as i32 - 1) {
            put_pixel(buf, width, height, x, y, color);
        }
    }
}

fn fill_tile_block(buf: &mut [u8], width: usize, height: usize, px: i32, py: i32, color: [u8; 4]) {
    let x0 = px;
    let y0 = py;
    let x1 = px + SCHEMATIC_TILE_PIXELS as i32 - 1;
    let y1 = py + SCHEMATIC_TILE_PIXELS as i32 - 1;
    fill_rect(buf, width, height, x0, y0, x1, y1, color);
}

fn new_schematic_image(width: usize, height: usize, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn render_placeholder_image(width: usize, height: usize) -> Image {
    let mut buf = vec![0u8; width * height * 4];
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let rand = ((x * 31 + y * 17) % 3) as u8;
            let color = [rand * 70 + 30, rand * 70 + 30, rand * 70 + 30, 255];
            put_pixel(&mut buf, width, height, x, y, color);
        }
    }

    new_schematic_image(width, height, buf)
}

fn render_schematic_image(
    layer: SchematicLayer,
    graph: &RoomGraph,
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
    tile_data: &[(&GridCoordinates, &TileNode, &Transform)],
    time_seconds: f32,
) -> Image {
    let mut buf = vec![0u8; width * height * 4];

    match layer {
        SchematicLayer::Electrical => {
            render_electrical_image(
                graph, &mut buf, width, height, min_x, max_y, tile_data,
            );
        }
        SchematicLayer::Fluidics => {
            render_fluidics_image(
                graph, &mut buf, width, height, min_x, max_y, tile_data,
            );
        }
        SchematicLayer::Atmospherics => {
            render_atmospherics_image(
                graph, &mut buf, width, height, min_x, max_y, tile_data,
            );
        }
        SchematicLayer::Thermal => {
            render_thermal_image(
                graph, &mut buf, width, height, min_x, max_y, tile_data,
            );
        }
        SchematicLayer::Research => {
            render_research_image(
                graph, &mut buf, width, height, min_x, max_y,
            );
        }
        SchematicLayer::Docking => {
            render_docking_image(
                graph, &mut buf, width, height, min_x, max_y,
            );
        }
        SchematicLayer::Sensors => {
            render_sensors_image(
                graph, &mut buf, width, height, min_x, max_y, time_seconds,
            );
        }
        _ => {
            render_reserved_image(&mut buf, width, height);
        }
    }

    new_schematic_image(width, height, buf)
}

fn world_to_schematic_pixel(
    grid_x: f32,
    grid_y: f32,
    min_x: i32,
    max_y: i32,
) -> Option<(i32, i32)> {
    let px = (grid_x.floor() as i32 - min_x) * SCHEMATIC_TILE_PIXELS as i32;
    let py = (max_y - grid_y.floor() as i32) * SCHEMATIC_TILE_PIXELS as i32;
    Some((px, py))
}

fn render_electrical_image(
    graph: &RoomGraph,
    buf: &mut [u8],
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
    tile_data: &[(&GridCoordinates, &TileNode, &Transform)],
) {
    let dim_color = [8u8, 12, 16, 255];
    let conduit_color = [0u8, 255, 255, 255];
    let node_color = [255u8, 255, 0, 255];
    let node_radius = 2 * SCHEMATIC_TILE_PIXELS as i32;

    for i in (0..buf.len()).step_by(4) {
        buf[i] = dim_color[0];
        buf[i + 1] = dim_color[1];
        buf[i + 2] = dim_color[2];
        buf[i + 3] = 255;
    }

    for (coords, tile, _) in tile_data.iter() {
        if tile.tile_type == TileType::Conduit {
            if let Some((px, py)) =
                world_to_schematic_pixel(coords.x as f32, coords.y as f32, min_x, max_y)
            {
                fill_tile_block(buf, width, height, px, py, conduit_color);
            }
        }
    }

    for room in graph.nodes() {
        if room.system_flags.contains(SystemFlags::ELECTRICAL) {
            let (cx, cy) = room_center(room);
            if let Some((px, py)) =
                world_to_schematic_pixel(cx, cy, min_x, max_y)
            {
                draw_circle(buf, width, height, px, py, node_radius, node_color);
            }
        }
    }

    for edge in graph.edges() {
        let (Some(from), Some(to)) = (graph.get_node(edge.from), graph.get_node(edge.to)) else {
            continue;
        };

        if from.system_flags.contains(SystemFlags::ELECTRICAL)
            && to.system_flags.contains(SystemFlags::ELECTRICAL)
        {
            let (fx, fy) = room_center(from);
            let (tx, ty) = room_center(to);
            if let Some((x0, y0)) = world_to_schematic_pixel(fx, fy, min_x, max_y) {
                if let Some((x1, y1)) = world_to_schematic_pixel(tx, ty, min_x, max_y) {
                    draw_line(buf, width, height, x0, y0, x1, y1, conduit_color);
                }
            }
        }
    }
}

fn render_fluidics_image(
    graph: &RoomGraph,
    buf: &mut [u8],
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
    tile_data: &[(&GridCoordinates, &TileNode, &Transform)],
) {
    let floor_color = [20u8, 22, 20, 255];
    let pipe_color = [0u8, 210, 255, 255];
    let wall_color = [5u8, 10, 5, 255];
    let tank_color = [0u8, 255, 150, 255];

    for i in (0..buf.len()).step_by(4) {
        buf[i] = floor_color[0];
        buf[i + 1] = floor_color[1];
        buf[i + 2] = floor_color[2];
        buf[i + 3] = 255;
    }

    for (coords, tile, _) in tile_data.iter() {
        if let Some((px, py)) =
            world_to_schematic_pixel(coords.x as f32, coords.y as f32, min_x, max_y)
        {
            let color = match tile.tile_type {
                TileType::Pipe => pipe_color,
                TileType::Wall(_) => wall_color,
                TileType::Door(_) => tank_color,
                TileType::Conduit => [0u8, 160, 255, 255],
                TileType::Floor => floor_color,
                TileType::Void => floor_color,
            };
            fill_tile_block(buf, width, height, px, py, color);
        }
    }

    for edge in graph.edges() {
        let (Some(from), Some(to)) = (graph.get_node(edge.from), graph.get_node(edge.to)) else {
            continue;
        };
        if from.system_flags.contains(SystemFlags::FLUIDICS)
            && to.system_flags.contains(SystemFlags::FLUIDICS)
        {
            let (fx, fy) = room_center(from);
            let (tx, ty) = room_center(to);
            if let Some((x0, y0)) = world_to_schematic_pixel(fx, fy, min_x, max_y) {
                if let Some((x1, y1)) = world_to_schematic_pixel(tx, ty, min_x, max_y) {
                    draw_line(buf, width, height, x0, y0, x1, y1, pipe_color);
                }
            }
        }
    }
}

fn render_atmospherics_image(
    _graph: &RoomGraph,
    buf: &mut [u8],
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
    tile_data: &[(&GridCoordinates, &TileNode, &Transform)],
) {
    let warm_color = [180u8, 120, 20, 255];
    let cool_color = [20u8, 150, 190, 255];
    let wall_color = [25u8, 30, 50, 255];

    for i in (0..buf.len()).step_by(4) {
        buf[i] = cool_color[0];
        buf[i + 1] = cool_color[1];
        buf[i + 2] = cool_color[2];
        buf[i + 3] = 255;
    }

    for (coords, tile, _) in tile_data.iter() {
        if let Some((px, py)) =
            world_to_schematic_pixel(coords.x as f32, coords.y as f32, min_x, max_y)
        {
            let color = match tile.tile_type {
                TileType::Wall(_) => wall_color,
                TileType::Door(_) => warm_color,
                TileType::Conduit => [255u8, 200, 0, 255],
                TileType::Pipe => [255u8, 0, 0, 255],
                TileType::Floor => cool_color,
                TileType::Void => cool_color,
            };
            fill_tile_block(buf, width, height, px, py, color);
        }
    }
}

fn render_thermal_image(
    graph: &RoomGraph,
    buf: &mut [u8],
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
    tile_data: &[(&GridCoordinates, &TileNode, &Transform)],
) {
    let cool = [10u8, 20, 90, 255];
    let hot = [255u8, 40, 0, 255];
    let very_hot = [255u8, 255, 200, 255];
    let hot_radius = 3 * SCHEMATIC_TILE_PIXELS as i32;

    for i in (0..buf.len()).step_by(4) {
        buf[i] = cool[0];
        buf[i + 1] = cool[1];
        buf[i + 2] = cool[2];
        buf[i + 3] = 255;
    }

    for (coords, tile, _) in tile_data.iter() {
        if let Some((px, py)) =
            world_to_schematic_pixel(coords.x as f32, coords.y as f32, min_x, max_y)
        {
            let color = match tile.tile_type {
                TileType::Conduit => very_hot,
                TileType::Pipe => hot,
                _ => {
                    if tile.tile_type.is_wall() {
                        [30u8, 15, 10, 255]
                    } else {
                        cool
                    }
                }
            };
            fill_tile_block(buf, width, height, px, py, color);
        }
    }

    for room in graph.nodes() {
        if room.system_flags.contains(SystemFlags::THERMAL) {
            let (cx, cy) = room_center(room);
            if let Some((px, py)) = world_to_schematic_pixel(cx, cy, min_x, max_y) {
                draw_circle(buf, width, height, px, py, hot_radius, very_hot);
            }
        }
    }
}

fn render_research_image(
    graph: &RoomGraph,
    buf: &mut [u8],
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
) {
    let bg = [10u8, 8, 30, 255];
    let line_color = [255u8, 215, 0, 255];
    let node_color = [255u8, 255, 255, 255];
    let node_radius = 3 * SCHEMATIC_TILE_PIXELS as i32;

    for i in (0..buf.len()).step_by(4) {
        buf[i] = bg[0];
        buf[i + 1] = bg[1];
        buf[i + 2] = bg[2];
        buf[i + 3] = 255;
    }

    for edge in graph.edges() {
        let (Some(from), Some(to)) = (graph.get_node(edge.from), graph.get_node(edge.to)) else {
            continue;
        };
        let (fx, fy) = room_center(from);
        let (tx, ty) = room_center(to);
        if let Some((x0, y0)) = world_to_schematic_pixel(fx, fy, min_x, max_y) {
            if let Some((x1, y1)) = world_to_schematic_pixel(tx, ty, min_x, max_y) {
                draw_line(buf, width, height, x0, y0, x1, y1, line_color);
            }
        }
    }

    for room in graph.nodes() {
        let (cx, cy) = room_center(room);
        if let Some((px, py)) = world_to_schematic_pixel(cx, cy, min_x, max_y) {
            draw_circle(buf, width, height, px, py, node_radius, node_color);
        }
    }
}

fn render_docking_image(
    graph: &RoomGraph,
    buf: &mut [u8],
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
) {
    let bg = [0u8, 20, 60, 255];
    let dashed = [255u8, 255, 255, 255];
    let airlock = [180u8, 255, 0, 255];

    for i in (0..buf.len()).step_by(4) {
        buf[i] = bg[0];
        buf[i + 1] = bg[1];
        buf[i + 2] = bg[2];
        buf[i + 3] = 255;
    }

    for room in graph.nodes() {
        if room.room_type == RoomType::Airlock {
            let (cx, cy) = room_center(room);
            if let Some((px, py)) = world_to_schematic_pixel(cx, cy, min_x, max_y) {
                fill_tile_block(buf, width, height, px, py, airlock);
            }
        }
    }

    for edge in graph.edges() {
        if edge.kind == EdgeKind::Airlock || edge.kind == EdgeKind::Umbilical {
            let (Some(from), Some(to)) = (graph.get_node(edge.from), graph.get_node(edge.to)) else {
                continue;
            };
            let (fx, fy) = room_center(from);
            let (tx, ty) = room_center(to);
            if let Some((x0, y0)) = world_to_schematic_pixel(fx, fy, min_x, max_y) {
                if let Some((x1, y1)) = world_to_schematic_pixel(tx, ty, min_x, max_y) {
                    draw_line(buf, width, height, x0, y0, x1, y1, dashed);
                }
            }
        }
    }
}

fn render_sensors_image(
    _graph: &RoomGraph,
    buf: &mut [u8],
    width: usize,
    height: usize,
    min_x: i32,
    max_y: i32,
    time_seconds: f32,
) {
    let bg = [0u8, 8, 10, 255];
    let sweep_color = [0u8, 255, 80, 255];
    let ping_color = [255u8, 40, 40, 255];

    for i in (0..buf.len()).step_by(4) {
        buf[i] = bg[0];
        buf[i + 1] = bg[1];
        buf[i + 2] = bg[2];
        buf[i + 3] = 255;
    }

    let center_x = width as i32 / 2;
    let center_y = height as i32 / 2;
    let radius = width.min(height) as i32 / 2;

    draw_circle(buf, width, height, center_x, center_y, radius, [0u8, 40, 20, 255]);

    let angle = (time_seconds * 0.8).rem_euclid(std::f32::consts::TAU);
    let dx = (angle.cos() * radius as f32) as i32;
    let dy = (angle.sin() * radius as f32) as i32;
    draw_line(buf, width, height, center_x, center_y, center_x + dx, center_y + dy, sweep_color);

    for edge in _graph.edges() {
        let (Some(from), Some(to)) = (_graph.get_node(edge.from), _graph.get_node(edge.to)) else {
            continue;
        };
        let (fx, fy) = room_center(from);
        let (tx, ty) = room_center(to);
        if let Some((x0, y0)) = world_to_schematic_pixel(fx, fy, min_x, max_y) {
            if let Some((x1, y1)) = world_to_schematic_pixel(tx, ty, min_x, max_y) {
                draw_line(buf, width, height, x0, y0, x1, y1, ping_color);
            }
        }
    }
}

fn render_reserved_image(buf: &mut [u8], width: usize, height: usize) {
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let noise = ((x * 1103515245 + y * 12345 + 1) % 256) as u8;
            let color = [
                noise.saturating_mul(1),
                noise.saturating_mul(1),
                noise.saturating_mul(1),
                255,
            ];
            put_pixel(buf, width, height, x, y, color);
        }
    }
}

fn render_terminal_transcript(graph: &RoomGraph) -> String {
    let mut output = String::from("$ netmap --live\n");
    for room in graph.nodes() {
        let status = if room.system_flags.is_empty() { "DOWN" } else { "UP" };
        output.push_str(&format!(
            "> {:>12}    [{}]\n",
            format!("{:?}", room.room_type),
            status
        ));
    }

    output.push_str("$ trace route --all\n");
    for edge in graph.edges() {
        if let (Some(a), Some(b)) = (graph.get_node(edge.from), graph.get_node(edge.to)) {
            output.push_str(&format!(
                "  {:?} -> {:?}\n",
                a.room_type, b.room_type
            ));
        }
    }

    output.push_str("> _");
    output
}

pub fn update_schematic_legend(
    active_layers: Res<SchematicLayers>,
    mut text_query: Query<&mut Text, With<SchematicLegendText>>,
) {
    const LAYERS: [(SchematicLayer, &str); 12] = [
        (SchematicLayer::Structural, "Structural"),
        (SchematicLayer::Electrical, "Electrical"),
        (SchematicLayer::Fluidics, "Fluidics"),
        (SchematicLayer::Atmospherics, "Atmospherics"),
        (SchematicLayer::Thermal, "Thermal"),
        (SchematicLayer::Computer, "Computer/Network"),
        (SchematicLayer::Research, "Research/Tech"),
        (SchematicLayer::Docking, "Docking/Travel"),
        (SchematicLayer::Sensors, "Sensors/Tactical"),
        (SchematicLayer::Reserved, "Reserved"),
        (SchematicLayer::Reserved, "Reserved"),
        (SchematicLayer::Reserved, "Reserved"),
    ];

    let mut active_names = Vec::new();
    for (layer, name) in LAYERS.iter() {
        if active_layers.active.contains(layer) {
            active_names.push(*name);
        }
    }

    let value = if active_names.is_empty() {
        "No layers active".to_string()
    } else {
        format!("Active layers: {}", active_names.join(", "))
    };

    for mut text in text_query.iter_mut() {
        text.sections[0].value = value.clone();
    }
}
