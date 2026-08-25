use bevy::{camera::Camera3d, ecs::resource::Resource, math::Vec2, utils::default};

const SENSITIVITY: f32 = 0.5;
const ZOOM_LIMIT: f32 = 2.0;

#[derive(Default, Clone)]
pub struct WorldCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Default, Clone)]
pub struct SphericalCoordinates {
    pub r: f32,
    pub theta: f32,
    pub phi: f32,
}

#[derive(Resource, Clone)]
pub struct Camera {
    pub sphere_coords: SphericalCoordinates,
    pub world_coords: WorldCoordinates,
    pub current_layer: usize,
    pub max_layer: usize,
    camera3d: Camera3d,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            sphere_coords: SphericalCoordinates::default(),
            world_coords: WorldCoordinates::default(),
            current_layer: 0,
            max_layer: 1,
            camera3d: Camera3d::default(),
        }
    }
}

impl Camera {
    pub fn new(cube: &usize, max_layer: usize) -> Self {
        // TODO: not hardcode these values
        let world_coords = WorldCoordinates {
            x: 40.0,
            y: -10.0,
            z: 0.0,
        };
        let (_, theta, phi) =
            Self::convert_cartesian_sphere(&world_coords.x, &world_coords.y, &world_coords.z);
        let r = *cube as f32 * 2.0;
        let sphere_coords = SphericalCoordinates { r, theta, phi };
        Self {
            sphere_coords,
            world_coords,
            current_layer: 0,
            max_layer,
            ..default()
        }
    }

    pub fn move_camera(&mut self, delta: Vec2) {
        self.sphere_coords.theta += delta.x * SENSITIVITY;
        self.sphere_coords.phi =
            (self.sphere_coords.phi - delta.y * SENSITIVITY).clamp(0.01, 179.99);
    }

    pub fn scroll_camera(&mut self, scroll: f32) {
        self.sphere_coords.r = scroll;
    }

    pub fn update_world_coords(&mut self) {
        let (x, y, z) = Self::convert_sphere_cartesian(
            &self.sphere_coords.r,
            &self.sphere_coords.theta,
            &self.sphere_coords.phi,
        );
        self.world_coords.x = x;
        self.world_coords.y = y;
        self.world_coords.z = z;
    }

    pub fn convert_sphere_cartesian(r: &f32, theta: &f32, phi: &f32) -> (f32, f32, f32) {
        let r = r.to_radians();
        let theta = theta.to_radians();
        let phi = phi.to_radians();
        let x = r * phi.sin() * theta.cos();
        let y = r * phi.cos();
        let z = r * theta.sin() * phi.sin();
        (x.to_degrees(), y.to_degrees(), z.to_degrees())
    }

    pub fn convert_cartesian_sphere(x: &f32, y: &f32, z: &f32) -> (f32, f32, f32) {
        let x = x.to_radians();
        let y = y.to_radians();
        let z = z.to_radians();
        let r: f32 = (x * x + y * y + z * z).sqrt();
        let theta: f32 = (z / r).acos();
        let phi: f32 = y.atan2(x);
        (r, theta, phi)
    }
}
