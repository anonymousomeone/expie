use glam::{Mat4, Vec2};
use winit::dpi::PhysicalSize;

pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Camera {

        Camera {
            position: Vec2::new(0.0, 0.0),
            zoom: 0.25,
        }
    }

    pub fn screen_to_world_2d(&self, pos: Vec2, screen_size: Vec2) -> Vec2 {
        let height = 1.0 / self.zoom;
        let width = height * (screen_size.x / screen_size.y);

        let ndc = Vec2::new(
            (pos.x / screen_size.x) * 2.0 - 1.0,
            1.0 - (pos.y / screen_size.y) * 2.0,
        );

        ndc * Vec2::new(width, height) + self.position
    }
}

pub struct CameraController {
    pub controller_type: ControllerType
}

impl CameraController {
    pub fn new(controller_type: ControllerType) -> CameraController {
        CameraController {
            controller_type,
        }
    }
}

pub enum ControllerType {
    Freecam,
}