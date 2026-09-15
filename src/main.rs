use std::collections::{HashMap, HashSet};

use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    color::palettes::css::GREY,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    input::mouse::MouseWheel,
    prelude::*,
    render::render_resource::{self, TextureFormat, TextureUsages},
    text::{EditableText, EditableTextFilter, TextCursorStyle},
};

use crate::{
    camera::Camera,
    cube::{Cube, CubeOpener, CubeSpawner},
    game::Game,
    ui::{bomb_display, text_submission},
};

mod camera;
mod cube;
mod game;
mod ui;

const DEFAULT_CUBES: usize = 3;
const CUBES_SPAWN_PER_FRAME: usize = 5;
const CUBES_OPEN_PER_FRAME: usize = 100;

struct GameColours;

impl GameColours {
    const DEFAULT_LIGHT: Color =
        Color::Srgba(Srgba::new(146. / 256., 177. / 256., 81. / 256., 256.));
    const DEFAULT_DARK: Color =
        Color::Srgba(Srgba::new(140. / 256., 171. / 256., 75. / 256., 256.));
    const REVEALED_LIGHT: Color =
        Color::Srgba(Srgba::new(110. / 256., 148. / 256., 39. / 256., 256.));
    const REVEALED_DARK: Color =
        Color::Srgba(Srgba::new(104. / 256., 142. / 256., 33. / 256., 256.));
    const FLAGGED: Color = Color::Srgba(Srgba::new(200. / 256., 200. / 256., 35. / 256., 256.));
    const BOMB: Color = Color::Srgba(Srgba::new(250. / 256., 144. / 256., 80. / 256., 256.));
}

#[derive(Resource, Default)]
struct CubeIndex(HashMap<(usize, usize, usize), Entity>);

#[derive(Component)]
struct SurfaceText(Entity);

#[derive(Component)]
struct SurfaceBackground(Entity);

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct CubeInput;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
}

fn main() {
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
                        enabled: true,
                        min_fps: 30.0,
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .init_state::<GameState>()
        .init_resource::<CubeIndex>()
        .add_systems(
            Startup,
            (
                spawn_light,
                spawn_camera3d,
                warmup_pipeline,
            )
                .chain(),
        )
        .add_systems(OnEnter(GameState::MainMenu), main_menu)
        .add_systems(
            Update,
            text_submission.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(
            Update,
            spawn_cubes.run_if(
                in_state(GameState::Playing)
                    .and_then(|spawner: Res<CubeSpawner>| spawner.spawned < spawner.to_spawn),
            ),
        )
        .add_systems(
            Update,
            update_cubes.run_if(
                in_state(GameState::Playing)
                    .and_then(|opener: Res<CubeOpener>| opener.opened < opener.to_open.len()),
            ),
        )
        .add_systems(
            Update,
            (
                bomb_display,
                scroll,
                movement,
                update_camera,
                manage_texture_cameras,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn update_cubes(
    game: Res<Game>,
    mut game_state: ResMut<NextState<GameState>>,
    mut cube_opener: ResMut<CubeOpener>,
    cube_index: Res<CubeIndex>,
    mut text_query: Query<(&mut Text, &SurfaceText)>,
    mut bg_query: Query<(&SurfaceBackground, &mut BackgroundColor)>,
    cube_query: Query<&Cube>,
) {
    for _ in 0..CUBES_OPEN_PER_FRAME {
        if cube_opener.opened >= cube_opener.to_open.len() {
            cube_opener.to_open = Vec::new();
            cube_opener.opened = 0;
            break;
        }
        info!("this gets fired");

        let (x, y, z) = cube_opener.to_open[cube_opener.opened];
        if let Some(&entity) = cube_index.0.get(&(x, y, z))
            && let Some(block) = game.get_block(x, y, z)
            && let Ok(cube) = cube_query.get(entity)
        {
            for (SurfaceBackground(bg_entity), mut bg_colour) in &mut bg_query {
                if entity == *bg_entity {
                    let is_white = (x + y + z).is_multiple_of(2);
                    if block.is_bomb && block.is_revealed {
                        bg_colour.0 = GameColours::BOMB;
                        game_state.set(GameState::MainMenu);
                    } else if cube.is_flagged {
                        bg_colour.0 = GameColours::FLAGGED;
                    } else if block.is_revealed {
                        bg_colour.0 = if is_white {
                            GameColours::REVEALED_LIGHT
                        } else {
                            GameColours::REVEALED_DARK
                        };
                        for (mut text, SurfaceText(text_cube_entity)) in &mut text_query {
                            if *text_cube_entity == entity {
                                text.0 = format!("{}", block.nearby_bombs);
                                break;
                            }
                        }
                    } else {
                        bg_colour.0 = if is_white {
                            GameColours::DEFAULT_LIGHT
                        } else {
                            GameColours::DEFAULT_DARK
                        };
                    }
                }
            }
        };

        cube_opener.opened += 1;
    }
    let win = game.check_win();
    if win {
        game_state.set(GameState::MainMenu);
    }
}

fn warmup_pipeline(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let size = render_resource::Extent3d {
        width: 4,
        height: 4,
        ..Default::default()
    };
    let mut image = Image::new_fill(
        size,
        render_resource::TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.0,
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..Default::default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.0, -10000.0, 0.0),
    ));

    let blend_material = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(blend_material),
        Transform::from_xyz(0.0, -10000.0, 0.0),
    ));

    let warm_size = render_resource::Extent3d {
        width: 256,
        height: 256,
        ..Default::default()
    };
    let mut warm_image = Image::new_fill(
        warm_size,
        render_resource::TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    warm_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let warm_image_handle = images.add(warm_image);

    let warm_camera = commands
        .spawn((
            Camera2d,
            bevy::camera::Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..Default::default()
            },
            RenderTarget::Image(warm_image_handle.into()),
        ))
        .id();

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::WHITE),
            UiTargetCamera(warm_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("0123456789"), // force every digit glyph to rasterize now
                TextFont {
                    font_size: FontSize::Px(50.0),
                    ..Default::default()
                },
                TextColor::BLACK,
            ));
        });

    game_state.set(GameState::MainMenu);
}

fn spawn_camera3d(mut commands: Commands) {
    let camera = Camera::new(&1);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            camera.world_coords.x,
            camera.world_coords.y,
            camera.world_coords.z,
        )
        .looking_at(Vec3::ZERO, Vec3::Y),
        camera,
    ));
}

fn spawn_light(mut commands: Commands) {
    commands.spawn((
        PointLight {
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn manage_texture_cameras(
    mut initial_render_done: Local<HashSet<Entity>>,
    cube_query: Query<(Entity, &Cube)>,
    text_query: Query<&SurfaceText, Changed<Text>>,
    bg_query: Query<&SurfaceBackground, Changed<BackgroundColor>>,
    mut camera_query: Query<&mut bevy::camera::Camera, With<Camera2d>>,
) {
    let mut needs_render: HashSet<Entity> = HashSet::new();
    for SurfaceText(cube_entity) in &text_query {
        needs_render.insert(*cube_entity);
    }
    for SurfaceBackground(cube_entity) in &bg_query {
        needs_render.insert(*cube_entity);
    }

    for (cube_entity, cube) in &cube_query {
        let first_time = initial_render_done.insert(cube_entity); // true if newly inserted
        let should_render = first_time || needs_render.contains(&cube_entity);

        if let Ok(mut camera) = camera_query.get_mut(cube.texture_camera) {
            camera.is_active = should_render;
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
    mut camera: ResMut<Camera>,
) {
    let mut layer_changed = false;
    for wheel in input.read() {
        if wheel.y > 0.0 && game.current_layer < game.max_layer {
            game.current_layer += 1;
            layer_changed = true;
        } else if wheel.y < 0.0 && game.current_layer > 0 {
            game.current_layer -= 1;
            layer_changed = true;
        }
        camera.scroll_camera((game.x - game.current_layer) * 3);
    }

    if !layer_changed {
        return;
    }

    for (mut cube, material, mut pickable) in &mut cube_query {
        cube.is_selectable = cube.layer == game.current_layer;
        let dim = cube.layer < game.current_layer;

        if dim == cube.is_dimmed {
            continue;
        }

        cube.is_dimmed = dim;

        if let Some(mut material) = materials.get_mut(&material.0) {
            material.base_color.set_alpha(if dim { 0.1 } else { 1.0 });
            material.alpha_mode = if dim {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
        }
        *pickable = if dim {
            Pickable::IGNORE
        } else {
            Pickable::default()
        };
    }
}

fn spawn_cubes(
    mut commands: Commands,
    game: Res<Game>,
    mut cube_spawner: ResMut<CubeSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cube_index: ResMut<CubeIndex>,
) {
    let shared_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    for _ in 0..CUBES_SPAWN_PER_FRAME {
        if cube_spawner.spawned >= cube_spawner.to_spawn {
            break;
        }

        let i = cube_spawner.spawned;
        let cube_per_layer = game.x;

        let row = i % cube_per_layer;
        let height = (i / cube_per_layer) % cube_per_layer;
        let depth = i / (cube_per_layer * cube_per_layer);

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
        let size = render_resource::Extent3d {
            width: 256,
            height: 256,
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
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..Default::default()
                },
                RenderTarget::Image(image_handle.clone().into()),
                DespawnOnExit(GameState::Playing),
            ))
            .id();
        let cube_entity = commands.spawn_empty().id();
        cube_index.0.insert((row, height, depth), cube_entity);
        let is_white = (row + height + depth).is_multiple_of(2);
        let bg_colour = if is_white {
            GameColours::DEFAULT_LIGHT
        } else {
            GameColours::DEFAULT_DARK
        };
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
                BackgroundColor(bg_colour),
                UiTargetCamera(texture_camera),
            ))
            .with_children(|parent| {
                parent.spawn((Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    ..default()
                },));
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
            alpha_mode: AlphaMode::Opaque,
            unlit: true,
            ..Default::default()
        });
        commands
            .entity(cube_entity)
            .insert((
                Cube {
                    row,
                    height,
                    depth,
                    layer,
                    is_selectable,
                    texture_camera,
                    is_dimmed: false,
                    is_flagged: false,
                },
                Pickable::default(),
                Mesh3d(shared_mesh.clone()),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(pos_x, pos_y, pos_z),
            ))
            .observe(
                move |click: On<Pointer<Click>>,
                      mut cube_query: Query<&mut Cube>,
                      mut game: ResMut<Game>,
                      mut cube_opener: ResMut<CubeOpener>| match click.button {
                    PointerButton::Primary => {
                        if let Ok(cube) = cube_query.get(click.entity)
                            && cube.is_selectable
                            && !cube.is_flagged
                            && let Some(opened_blocks) =
                                game.open(cube.row, cube.height, cube.depth)
                        {
                            cube_opener.to_open = opened_blocks;
                            cube_opener.opened = 0;
                        }
                    }
                    PointerButton::Secondary => {}
                    PointerButton::Middle => {
                        if let Ok(mut cube) = cube_query.get_mut(click.entity)
                            && let Some(block) = game.get_block(cube.row, cube.height, cube.depth)
                            && !block.is_revealed
                        {
                            cube.is_flagged = !cube.is_flagged;
                            cube_opener
                                .to_open
                                .push((cube.row, cube.height, cube.depth));
                        }
                    }
                },
            );
        cube_spawner.spawned += 1;
    }
}

fn main_menu(mut commands: Commands, game: Option<Res<Game>>) {
    let mut cube = DEFAULT_CUBES;
    if let Some(game) = game {
        cube = game.x;
    }

    let root = commands
        .spawn((
            MainMenuRoot,
            DespawnOnExit(GameState::Playing),
            Node {
                width: Val::Px(200.),
                height: Val::Px(50.),
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                border: UiRect::all(Val::Px(1.)),
                ..Default::default()
            },
        ))
        .id();
    let cube_label_box = commands
        .spawn((
            Node {
                height: Val::Percent(100.),
                width: Val::Percent(75.),
                align_items: AlignItems::Center,
                padding: Val::Px(1.).all(),
                border: Val::Px(2.).all(),
                ..Default::default()
            },
            BackgroundColor(GREY.into()),
            BorderColor::all(Color::WHITE),
        ))
        .id();
    let cube_label_text = commands
        .spawn((
            Text("Enter cube:".into()),
            TextFont {
                font_size: FontSize::Px(20.),
                ..Default::default()
            },
            TextColor(Color::BLACK),
        ))
        .id();
    let cube_input = commands
        .spawn((
            Node {
                width: Val::Percent(25.),
                height: Val::Percent(100.),
                align_self: AlignSelf::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                border: Val::Px(2.).all(),
                ..Default::default()
            },
            BackgroundColor(GREY.into()),
            BorderColor::all(Color::WHITE),
        ))
        .id();
    let mut a = EditableText::new(format!("{}", cube));
    a.max_characters = Some(1);
    a.visible_width = Some(10.);
    a.allow_newlines = false;
    let editable_text = commands
        .spawn((
            CubeInput,
            a,
            EditableTextFilter::new(|c| c.is_numeric()),
            TextLayout::no_wrap(),
            TextColor(Color::BLACK),
            TextCursorStyle::default(),
        ))
        .id();
    let label = commands
        .entity(cube_label_box)
        .add_children(&[cube_label_text])
        .id();
    let input = commands
        .entity(cube_input)
        .add_children(&[editable_text])
        .id();
    commands.entity(root).add_children(&[label, input]);
}
