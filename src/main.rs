use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::game::Game;

mod camera;
mod cube;
mod game;
mod moving;

#[derive(Resource)]
struct SphericalCoordinates {
    r: f32,
    theta: f32,
    phi: f32,
}

// enum LockCursor {
//     Yes,
//     No,
// }

const SENSITIVITY: f32 = 0.5;
const ZOOM_LIMIT: f32 = 2.0;

struct OverlayColor;

impl OverlayColor {
    // const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

fn main() {
    let game = Game::new(3, 3, 3, 4); // might need mut

    let (_, theta, phi) = convert_cartesian_sphere(40.0, -10.0, 0.0);
    let r = 10.0;
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
                        enabled: true,
                        min_fps: 30.0,
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .insert_resource(SphericalCoordinates { r, theta, phi })
        .insert_resource(game)
        // .add_systems(Startup, scene.spawn())
        .add_systems(Startup, spawn_scene)
        // .add_systems(Update, draw_cube)
        .add_systems(Update, scroll)
        .add_systems(Update, movement)
        .add_systems(Update, draw_cube_edges)
        .run();
}

// fn draw_cube(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let red = [1.0, 0.0, 0.0, 1.0];
//     let green = [0.0, 1.0, 0.0, 1.0];
//     let blue = [0.0, 0.0, 1.0, 1.0];
//     let yellow = [1.0, 1.0, 0.0, 1.0];
//     let orange = [1.0, 0.5, 0.0, 1.0];
//     let purple = [0.5, 0.0, 0.5, 1.0];
//
//     // 2. Generate the vertex array (4 vertices per face * 6 faces = 24 entries)
//     let vertex_colors = vec![
//         // Right face (4 vertices)
//         red, red, red, red, // Left face
//         green, green, green, green, // Top face
//         blue, blue, blue, blue, // Bottom face
//         yellow, yellow, yellow, yellow, // Forward face
//         orange, orange, orange, orange, // Back face
//         purple, purple, purple, purple,
//     ];
//
//     // 3. Create the mesh and insert the color attribute
//     let mut colorful_mesh = Mesh::from(Cuboid::default());
//     colorful_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
//
//     // 4. Spawn the cube with a white base material so colors aren't tinted
//     commands.spawn((
//         Mesh3d(meshes.add(colorful_mesh)),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             base_color: Color::from(LinearRgba::WHITE),
//             ..default()
//         })),
//         Transform::from_xyz(0.0, 0.0, 0.0),
//     ));
// }

fn draw_cube_edges(mut gizmos: Gizmos, query: Query<(&Name, &Transform)>) {
    for (name, transform) in &query {
        if name.as_str() == "Cube" {
            gizmos.cube(*transform, Color::WHITE);
        }
    }
}

fn movement(
    button_input: Res<ButtonInput<MouseButton>>,
    mut sphere_coords: ResMut<SphericalCoordinates>,
    mut move_input: MessageReader<CursorMoved>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    for mut transform in &mut query {
        if button_input.pressed(MouseButton::Right) {
            for message in move_input.read() {
                if let Some(delta) = message.delta {
                    sphere_coords.theta += delta.x * SENSITIVITY;
                    sphere_coords.phi =
                        (sphere_coords.phi - delta.y * SENSITIVITY).clamp(0.01, 179.99);
                }
            }
        }
        let (x, y, z) =
            convert_sphere_cartesian(sphere_coords.r, sphere_coords.theta, sphere_coords.phi);
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn scroll(
    mut input: MessageReader<MouseWheel>,
    mut sphere_coords: ResMut<SphericalCoordinates>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    for message in input.read() {
        let scroll = match message.unit {
            MouseScrollUnit::Line => message.y * 1.0,
            MouseScrollUnit::Pixel => message.y,
        };
        if scroll != 0.0 {
            info!("Scrolled {} units", scroll);
            for mut transform in &mut query {
                let x = transform.translation.x;
                let y = transform.translation.y;
                let z = transform.translation.z;
                info!("location: {}, {}, {}", x, y, z);
                info!(
                    "sphere location: {}, {}, {}",
                    sphere_coords.r, sphere_coords.theta, sphere_coords.phi
                );
                if sphere_coords.r - scroll < ZOOM_LIMIT {
                    sphere_coords.r = ZOOM_LIMIT;
                } else {
                    sphere_coords.r -= scroll;
                }
                let (x, y, z) = convert_sphere_cartesian(
                    sphere_coords.r,
                    sphere_coords.theta,
                    sphere_coords.phi,
                );
                transform.translation = Vec3::new(x, y, z);
            }
        }
    }
}

fn convert_sphere_cartesian(r: f32, theta: f32, phi: f32) -> (f32, f32, f32) {
    let r = r.to_radians();
    let theta = theta.to_radians();
    let phi = phi.to_radians();
    let x = r * phi.sin() * theta.cos();
    let y = r * phi.cos();
    let z = r * theta.sin() * phi.sin();
    (x.to_degrees(), y.to_degrees(), z.to_degrees())
}

fn convert_cartesian_sphere(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let x = x.to_radians();
    let y = y.to_radians();
    let z = z.to_radians();
    let r: f32 = (x * x + y * y + z * z).sqrt();
    let theta: f32 = (z / r).acos();
    let phi: f32 = y.atan2(x);
    (r, theta, phi)
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
                let pos_x = {
                    if row <= x / 2 {
                        -(row as f32)
                    } else {
                        row as f32 - 1.0
                    }
                };
                let pos_y = {
                    if height <= y / 2 {
                        -(height as f32)
                    } else {
                        height as f32 - 1.0
                    }
                };
                let pos_z = {
                    if depth <= z / 2 {
                        -(depth as f32)
                    } else {
                        depth as f32 - 1.0
                    }
                };
                cubes.push(bsn!(
                    #Cube
                    Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
                    // MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
                    Transform::from_xyz(pos_x, pos_y, pos_z)
                    on(move |click: On<Pointer<Click>>, mut game: ResMut<Game>| {
                        match click.button {
                            PointerButton::Primary => {
                                let mut block = game.get_block_mut(pos_x as usize, pos_y as usize, pos_z as usize).unwrap();


                                info!("cube clicked");
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
                Camera3d
                template_value(Transform::from_xyz(40.0, -10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y))
            ),
    )
}
