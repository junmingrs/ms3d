use bevy::ecs::{
    component::Component, entity::Entity, query::With, resource::Resource, system::{Query, ResMut, SystemParam}
};

use crate::CubeIndex;

#[derive(Resource)]
pub struct CubeSpawner {
    pub spawned: usize,
    pub to_spawn: usize,
}

#[derive(Resource)]
pub struct CubeOpener {
    pub opened: usize,
    pub to_open: Vec<(usize, usize, usize)>,
}

#[derive(SystemParam)]
pub struct CubeCleanup<'w, 's> {
    pub existing_cubes: Query<'w, 's, Entity, With<Cube>>,
    pub cube_index: ResMut<'w, CubeIndex>,
}

#[derive(Clone, Component)]
pub struct Cube {
    pub row: usize,
    pub height: usize,
    pub depth: usize,
    pub is_selectable: bool,
    pub layer: usize,
    pub texture_camera: Entity,
    pub is_dimmed: bool,
    pub is_flagged: bool,
}
