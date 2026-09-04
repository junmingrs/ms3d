use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    color::palettes::css::{GRAY, WHITE},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    input::mouse::MouseWheel,
    prelude::*,
    render::render_resource::{self, TextureFormat, TextureUsages},
};

use crate::{camera::Camera, game::Game};

mod camera;
mod cube;
mod game;
mod moving;

#[derive(Clone, Component, Default)]
struct Cube {
    x: f32,
    y: f32,
    z: f32,
    row: usize,
    height: usize,
    depth: usize,
    is_selectable: bool,
    is_selected: bool,
    is_hovered: bool,
    is_opened: bool,
    layer: usize,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct HoverGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct SelectableGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct NotSelectableGizmos;

#[derive(Component)]
struct SurfaceText(Entity);

#[derive(Component)]
struct SurfaceBackground(Entity);

fn main() {
    let cube = 3;
    let bombs = 3;
    let game = Game::new(cube, cube, cube, bombs);
    let camera = Camera::new(&cube, game.max_layer);

    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: FontSize::Px(42.0),
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: false,
                        min_fps: 30.0,
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .insert_resource(game)
        .insert_resource(camera)
        // .add_systems(Startup, scene.spawn())
        .add_systems(Startup, spawn_scene)
        .add_systems(Startup, setup_highlight_gizmo_config)
        .init_gizmo_group::<HoverGizmos>()
        .init_gizmo_group::<SelectableGizmos>()
        .init_gizmo_group::<NotSelectableGizmos>()
        .add_systems(Update, scroll)
        .add_systems(Update, movement)
        .add_systems(Update, update_camera)
        .add_systems(Update, update_text)
        // .add_systems(Update, draw_cube_edges)
        // .add_systems(Update, fade_cubes_near_camera)
        .run();
}

fn update_text(
    game: Res<Game>,
    mut query: Query<(&mut Text, &SurfaceText)>,
    cube_query: Query<&Cube>,
) {
    for (mut text, SurfaceText(cube_entity)) in &mut query {
        if let Ok(cube) = cube_query.get(*cube_entity)
            && let Some(block) = game.get_block(cube.row, cube.height, cube.depth)
            && block.is_revealed
        {
            text.0 = format!("{}", block.nearby_bombs);
        }
    }
}

fn update_camera(mut camera: ResMut<Camera>, mut query: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in &mut query {
        camera.update_world_coords();
        transform.translation = transform.translation.lerp(
            Vec3::new(
                camera.world_coords.x,
                camera.world_coords.y,
                camera.world_coords.z,
            ),
            0.1,
        );
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn setup_highlight_gizmo_config(mut config_store: ResMut<GizmoConfigStore>) {
    let (hover_config, _) = config_store.config_mut::<HoverGizmos>();
    hover_config.depth_bias = -1.0;
    let (selectable_config, _) = config_store.config_mut::<SelectableGizmos>();
    selectable_config.depth_bias = -0.5;
    let (not_selectable_config, _) = config_store.config_mut::<NotSelectableGizmos>();
    not_selectable_config.depth_bias = -0.1;
}

// fn draw_cube_edges(
//     // mut gizmos: Gizmos,
//     mut hover_gizmos: Gizmos<HoverGizmos>,
//     mut selectable_gizmos: Gizmos<SelectableGizmos>,
//     mut not_selectable_gizmos: Gizmos<NotSelectableGizmos>,
//     query: Query<(&Cube, &Transform)>,
//     game: Res<Game>,
// ) {
//     for (cube, transform) in &query {
//         let block = game.get_block(cube.row, cube.height, cube.depth).unwrap();
//         if cube.is_hovered {
//             hover_gizmos.cube(*transform, Color::srgb(0.9, 0.9, 0.9));
//         }
//         if cube.is_selected || cube.is_opened || block.is_revealed {
//             selectable_gizmos.cube(*transform, Color::WHITE);
//         } else {
//             if !cube.is_selectable || block.is_bomb {
//                 not_selectable_gizmos.cube(*transform, Color::srgb(1.0, 0.0, 0.0));
//             } else {
//                 selectable_gizmos.cube(*transform, Color::srgb(0.0, 1.0, 0.0));
//             }
//         }
//     }
// }

// fn fade_cubes_near_camera(
//     camera_query: Query<&Transform, With<Camera3d>>,
//     mut cube_query: Query<(&Transform, &mut Cube)>,
// ) {
//     let Ok(camera_transform) = camera_query.single() else {
//         return;
//     };
//
//     for (cube_transform, mut cube) in &mut cube_query {
//         let distance = camera_transform
//             .translation
//             .distance(cube_transform.translation);
//
//         if distance <= 4.0 {
//             cube.is_selectable = false;
//             cube.is_selected = false;
//         } else {
//             cube.is_selectable = true;
//         }
//     }
// }

fn movement(
    button_input: Res<ButtonInput<MouseButton>>,
    mut move_input: MessageReader<CursorMoved>,
    mut camera: ResMut<Camera>,
) {
    if button_input.pressed(MouseButton::Right) {
        for message in move_input.read() {
            if let Some(delta) = message.delta {
                camera.move_camera(delta);
            }
        }
    }
}

fn scroll(
    mut input: MessageReader<MouseWheel>,
    mut game: ResMut<Game>,
    mut cube_query: Query<(&mut Cube, &MeshMaterial3d<StandardMaterial>, &mut Pickable)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bg_query: Query<(&SurfaceBackground, &mut BackgroundColor)>,
    mut camera: ResMut<Camera>,
) {
    for wheel in input.read() {
        if wheel.y > 0.0 && game.current_layer < game.max_layer {
            game.current_layer += 1;
        } else if wheel.y < 0.0 && game.current_layer > 0 {
            game.current_layer -= 1;
        }
        camera.current_layer = game.current_layer;
        let scroll = (game.max_layer - game.current_layer + 5) as f32;
        camera.scroll_camera(scroll);
    }
    for (mut cube, material, mut pickable) in &mut cube_query {
        cube.is_selectable = cube.layer == game.current_layer;
        if !cube.is_selectable {
            cube.is_selected = false;
            cube.is_hovered = false;
        }
        if let Some(mut material) = materials.get_mut(&material.0) {
            let block = game.get_block(cube.row, cube.height, cube.depth).unwrap();
            if block.is_revealed {
                if block.is_bomb {
                    material.base_color = Color::Srgba(Srgba::rgba_u8(200, 100, 100, 100));
                } else {
                    material.base_color = Color::Srgba(Srgba::rgba_u8(100, 200, 100, 100));
                }
            }
            for (SurfaceBackground(_cube_entity), mut bg_colour) in &mut bg_query {
                if cube.layer < game.current_layer {
                    bg_colour.0.set_alpha(0.1);
                } else {
                    bg_colour.0.set_alpha(1.0);
                }
            }
            if cube.layer < game.current_layer {
                material.base_color.set_alpha(0.1);
            } else {
                material.base_color.set_alpha(1.0);
            }
        }
        *pickable = if cube.layer < game.current_layer {
            Pickable::IGNORE
        } else {
            Pickable::default()
        }
    }
}

fn spawn_scene(
    mut commands: Commands,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for depth in 0..game.z {
        for height in 0..game.y {
            for row in 0..game.x {
                let pos_x = row as f32 - (game.x as f32 - 1.0) / 2.0;
                let pos_y = height as f32 - (game.y as f32 - 1.0) / 2.0;
                let pos_z = depth as f32 - (game.z as f32 - 1.0) / 2.0;
                let layer = [
                    row,
                    game.x - 1 - row,
                    height,
                    game.y - 1 - height,
                    depth,
                    game.z - 1 - depth,
                ]
                .into_iter()
                .min()
                .unwrap();
                let is_selectable = layer == 0;
                // TODO: need get ui text on 3d cube
                let size = render_resource::Extent3d {
                    width: 512,
                    height: 512,
                    ..Default::default()
                };
                let mut image = Image::new_fill(
                    size,
                    render_resource::TextureDimension::D2,
                    &[0, 0, 0, 0],
                    TextureFormat::Bgra8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT;
                let image_handle = images.add(image);
                let texture_camera = commands
                    .spawn((
                        Camera2d,
                        bevy::camera::Camera {
                            order: -1,
                            ..Default::default()
                        },
                        RenderTarget::Image(image_handle.clone().into()),
                    ))
                    .id();
                let cube_entity = commands.spawn_empty().id();
                commands
                    .spawn((
                        SurfaceBackground(cube_entity),
                        Node {
                            width: percent(100),
                            height: percent(100),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        BackgroundColor(WHITE.into()),
                        UiTargetCamera(texture_camera),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            // SurfaceBackground(cube_entity),
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                align_items: AlignItems::Center,
                                // border_radius: BorderRadius::all(px(10.)),
                                ..default()
                            },
                            // BackgroundColor(WHITE.into()),
                        ));
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            SurfaceText(cube_entity),
                            Text::new(""),
                            TextFont {
                                font_size: FontSize::Px(50.0),
                                ..Default::default()
                            },
                            TextColor::BLACK,
                        ));
                    });
                let material_handle = materials.add(StandardMaterial {
                    base_color_texture: Some(image_handle),
                    reflectance: 0.0,
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                });
                commands
                    .entity(cube_entity)
                    .insert((
                        Cube {
                            x: pos_x,
                            y: pos_y,
                            z: pos_z,
                            row,
                            height,
                            depth,
                            layer,
                            is_selectable,
                            is_hovered: false,
                            is_selected: false,
                            is_opened: false,
                        },
                        Pickable::default(),
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(material_handle),
                        Transform::from_xyz(pos_x, pos_y, pos_z),
                    ))
                    .observe(
                        move |hover: On<Pointer<Enter>>, mut query: Query<&mut Cube>| {
                            if let Ok(mut cube) = query.get_mut(hover.entity)
                                && cube.is_selectable
                            {
                                cube.is_hovered = true;
                            }
                        },
                    )
                    .observe(
                        move |hover: On<Pointer<Leave>>, mut query: Query<&mut Cube>| {
                            if let Ok(mut cube) = query.get_mut(hover.entity)
                                && cube.is_selectable
                            {
                                cube.is_hovered = false;
                            }
                        },
                    )
                    .observe(
                        move |click: On<Pointer<Click>>,
                              mut query: Query<&mut Cube>,
                              mut game: ResMut<Game>| {
                            match click.button {
                                PointerButton::Primary => {
                                    if let Ok(mut cube) = query.get_mut(click.entity)
                                        && cube.is_selectable
                                    {
                                        // info!("cube clicked {}, {}, {}", pos_x, pos_y, pos_z);
                                        // cube.is_selected = true;
                                        // cube.is_opened = true;
                                        game.open(cube.row, cube.height, cube.depth);
                                    }
                                }
                                PointerButton::Secondary => {}
                                PointerButton::Middle => {}
                            }
                        },
                    );
            }
        }
    }
    commands.spawn((
        PointLight {
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(40.0, -10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
