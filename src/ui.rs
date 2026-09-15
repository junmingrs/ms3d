use std::collections::HashSet;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut, SystemParam},
    },
    input::{ButtonInput, keyboard::KeyCode},
    state::{state::NextState, state_scoped::DespawnOnExit},
    text::{EditableText, FontSize, TextColor, TextFont},
    ui::{Node, Val, widget::Text},
};

use crate::{
    CubeInput, DEFAULT_CUBES, GameState, InitialRender, MainMenuRoot,
    camera::Camera,
    cube::{CubeCleanup, CubeOpener, CubeSpawner},
    game::Game,
};

#[derive(Component)]
pub struct BombDisplay;

#[derive(SystemParam)]
pub struct MenuInput<'w, 's> {
    cube_input: Query<'w, 's, &'static EditableText, With<CubeInput>>,
    menu_root: Query<'w, 's, Entity, With<MainMenuRoot>>,
}

pub fn bomb_display(mut commands: Commands, game: Res<Game>) {
    commands.spawn((
        BombDisplay,
        Node {
            height: Val::Px(50.),
            width: Val::Px(200.),
            justify_self: bevy::ui::JustifySelf::End,
            align_self: bevy::ui::AlignSelf::Start,
            ..Default::default()
        },
        Text::new(format!("Bombs: {}", game.bombs)),
        TextFont {
            font_size: FontSize::Px(25.0),
            ..Default::default()
        },
        TextColor::WHITE,
        DespawnOnExit(GameState::GameEnd),
    ));
}

pub fn text_submission(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut camera: ResMut<Camera>,
    menu_input: MenuInput,
    mut cube_cleanup: CubeCleanup,
) {
    if let NextState::Pending(GameState::Playing) = game_state.as_ref() {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Enter)
        && let Ok(cube_input) = menu_input.cube_input.single()
    {
        let cube_str = cube_input.value().to_string();
        let cube_val: usize = if cube_str.is_empty() || cube_str == "0" {
            DEFAULT_CUBES
        } else {
            cube_str.parse().unwrap()
        };
        let total_cubes = cube_val.pow(3);
        let bombs_val = total_cubes / 5;
        let game = Game::new(cube_val, cube_val, cube_val, bombs_val);
        let cube_spawner = CubeSpawner {
            spawned: 0,
            to_spawn: total_cubes,
        };

        let cube_opener = CubeOpener {
            opened: 0,
            to_open: Vec::new(),
        };

        commands.insert_resource(game);
        commands.insert_resource(cube_spawner);
        commands.insert_resource(cube_opener);
        commands.insert_resource(InitialRender(HashSet::new()));
        camera.scroll_camera(cube_val * 3);

        for entity in &menu_input.menu_root {
            commands.entity(entity).despawn_children();
            commands.entity(entity).despawn();
        }

        for c in &cube_cleanup.existing_cubes {
            commands.entity(c).despawn();
        }
        cube_cleanup.cube_index.0.clear();

        game_state.set(GameState::Playing);
    }
}
