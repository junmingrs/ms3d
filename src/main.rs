use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    input::mouse::MouseWheel,
    prelude::*,
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
    is_selectable: bool,
    is_selected: bool,
    is_hovered: bool,
    layer: usize,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct HoverGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct SelectableGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct NotSelectableGizmos;

// #[derive(Clone)]
// enum Visibility {
//     Visible,
//     Hidden,
// }

// enum LockCursor {
//     Yes,
//     No,
// }

struct OverlayColor;

impl OverlayColor {
    // const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

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
                    text_color: OverlayColor::GREEN,
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
        .add_systems(Update, draw_cube_edges)
        // .add_systems(Update, fade_cubes_near_camera)
        .run();
}

fn update_camera(mut camera: ResMut<Camera>, mut query: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in &mut query {
        camera.update_world_coords();
        transform.translation = Vec3::new(
            camera.world_coords.x,
            camera.world_coords.y,
            camera.world_coords.z,
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
    not_selectable_config.depth_bias = 0.0;
}

fn draw_cube_edges(
    // mut gizmos: Gizmos,
    mut hover_gizmos: Gizmos<HoverGizmos>,
    mut selectable_gizmos: Gizmos<SelectableGizmos>,
    mut not_selectable_gizmos: Gizmos<NotSelectableGizmos>,
    query: Query<(&Cube, &Transform)>,
) {
    for (cube, transform) in &query {
        if cube.is_hovered {
            hover_gizmos.cube(*transform, Color::srgb(0.9, 0.9, 0.9));
        }
        if cube.is_selected {
            selectable_gizmos.cube(*transform, Color::WHITE);
        } else {
            if !cube.is_selectable {
                not_selectable_gizmos.cube(*transform, Color::srgb(1.0, 0.0, 0.0));
            } else {
                selectable_gizmos.cube(*transform, Color::srgb(0.0, 1.0, 0.0));
            }
        }
    }
}

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
    // mut sphere_coords: ResMut<SphericalCoordinates>,
    mut move_input: MessageReader<CursorMoved>,
    // mut query: Query<(&mut Transform, &mut Camera)>,
    mut camera: ResMut<Camera>,
) {
    if button_input.pressed(MouseButton::Right) {
        for message in move_input.read() {
            if let Some(delta) = message.delta {
                camera.move_camera(delta);
            }
        }
    }
    // for (mut transform, mut camera) in &mut query {
    //     if button_input.pressed(MouseButton::Right) {
    //         for message in move_input.read() {
    //             if let Some(delta) = message.delta {
    //                 camera.move_camera(delta);
    //                 //     sphere_coords.theta += delta.x * SENSITIVITY;
    //                 //     sphere_coords.phi =
    //                 //         (sphere_coords.phi - delta.y * SENSITIVITY).clamp(0.01, 179.99);
    //             }
    //         }
    //     }
    // let (x, y, z) =
    //     convert_sphere_cartesian(sphere_coords.r, sphere_coords.theta, sphere_coords.phi);
    // transform.translation = Vec3::new(x, y, z);
    // transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn scroll(
    time: Res<Time>,
    mut input: MessageReader<MouseWheel>,
    // mut sphere_coords: ResMut<SphericalCoordinates>,
    mut game: ResMut<Game>,
    mut cube_query: Query<(&mut Cube, &MeshMaterial3d<StandardMaterial>, &mut Pickable)>,
    // mut cube_query: Query<(&mut Cube, &mut Pickable)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // mut camera_query: Query<(&mut Transform, &mut Camera)>,
    mut camera: ResMut<Camera>,
    // mut query: Query<&mut Transform, With<Camera3d>>,
) {
    // if let Ok(mut transform) = query.single_mut() {
    for wheel in input.read() {
        if wheel.y > 0.0 && game.current_layer < game.max_layer {
            game.current_layer += 1;
            // info!("{}", game.current_layer);
        } else if wheel.y < 0.0 && game.current_layer > 0 {
            game.current_layer -= 1;
            // info!("{}", game.current_layer);
        }
        camera.current_layer = game.current_layer;
        let scroll = (game.max_layer - game.current_layer + 5) as f32;
        camera.scroll_camera(scroll);
        // camera.zoom_camera(delta_scroll);
        // camera.scroll_camera(scroll);
        // let (x, y, z) = Camera::convert_sphere_cartesian(
        //     &scroll,
        //     &camera.sphere_coords.theta,
        //     &camera.sphere_coords.phi,
        // );
        // transform.translation = transform.translation.lerp(Vec3::new(x, y, z), t);
        // }
    }
    for (mut cube, material, mut pickable) in &mut cube_query {
        // for (mut cube, mut pickable) in &mut cube_query {
        cube.is_selectable = cube.layer == game.current_layer;
        if !cube.is_selectable {
            cube.is_selected = false;
            cube.is_hovered = false;
        }
        if let Some(mut material) = materials.get_mut(&material.0) {
            if cube.layer < game.current_layer {
                material.base_color.set_alpha(0.1);
            } else {
                material.base_color.set_alpha(0.8);
            }
        }
        *pickable = if cube.layer < game.current_layer {
            Pickable::IGNORE
        } else {
            Pickable::default()
        }
    }
}

// fn normalize(x: f32, y: f32, z: f32) -> [f32; 3] {
//     let l = [x, y, z];
//     let mut res = [0.0; 3];
//     for r in res.iter_mut() {
//         let pre: f32 = (l[0] * l[0]) + (l[1] * l[1]) + (l[2] * l[2]);
//         *r = 1.0 / pre.sqrt();
//     }
//     res
// }

fn spawn_scene(mut commands: Commands, game: Res<Game>) {
    commands.spawn_scene_list(scene(game.x, game.y, game.z));
}

fn scene(x: usize, y: usize, z: usize) -> impl SceneList {
    let mut cubes = Vec::new();
    for depth in 0..z {
        for height in 0..y {
            for row in 0..x {
                let pos_x = row as f32 - (x as f32 - 1.0) / 2.0;
                let pos_y = height as f32 - (y as f32 - 1.0) / 2.0;
                let pos_z = depth as f32 - (z as f32 - 1.0) / 2.0;
                let layer = [
                    row,
                    x - 1 - row,
                    height,
                    y - 1 - height,
                    depth,
                    z - 1 - depth,
                ]
                .into_iter()
                .min()
                .unwrap();
                let is_selectable = layer == 0;
                cubes.push(bsn!(
                    Cube { x: pos_x, y: pos_y, z: pos_z, layer: layer, is_selectable, is_hovered: false, is_selected: false }
                    Pickable
                    Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(0.0, 1.0, 0.0, 0.8)))
                    Transform::from_xyz(pos_x, pos_y, pos_z)
                    on(move |hover: On<Pointer<Enter>>, mut query: Query<&mut Cube>| {
                        if let Ok(mut cube) = query.get_mut(hover.entity) && cube.is_selectable {
                            cube.is_hovered = true;
                        }
                    })
                    on(move |hover: On<Pointer<Leave>>, mut query: Query<&mut Cube>| {
                        if let Ok(mut cube) = query.get_mut(hover.entity) && cube.is_selectable {
                            cube.is_hovered = false;
                        }
                    })
                    on(move |click: On<Pointer<Click>>, mut query: Query<&mut Cube>| {
                        match click.button {
                            PointerButton::Primary => {
                                // let mut block = game.get_block_mut(pos_x as usize, pos_y as usize, pos_z as usize).unwrap();
                                if let Ok(mut cube) = query.get_mut(click.entity) && cube.is_selectable {
                                    // info!("cube clicked {}, {}, {}", pos_x, pos_y, pos_z);
                                    cube.is_selected = true;
                                }
                            },
                            PointerButton::Secondary => {},
                            PointerButton::Middle => {},
                        }
                    })
                ))
            }
        }
    }

    bsn_list!(
            {cubes}
            (
                PointLight {
                    shadow_maps_enabled: true,
                }
                Transform::from_xyz(4.0, -8.0, -4.0)
            ),
            (
                PointLight {
                    shadow_maps_enabled: true,
                }
                Transform::from_xyz(-4.0, 8.0, 4.0)
            ),
            (
                Camera
                Camera3d
                template_value(Transform::from_xyz(40.0, -10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y))
            ),
    )
}
