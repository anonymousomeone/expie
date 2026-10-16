use glam::{Mat4, Vec2, Vec3};

#[derive(Default)]
pub struct Camera {
    pub position: Vec2,
    pub viewport_size: Vec2,
    pub fov: f32,
}

impl Camera {
    pub fn to_matrix(&self, aspect_ratio: f32) -> Mat4 {
        // let translation = Mat4::IDENTITY * Mat4::from_translation(self.position.extend(1.0));
        let translation = Mat4::IDENTITY * Mat4::from_translation(Vec3::new(0.5, 0.5, 1.0));

        // let projection = glam::camera::rh::proj::directx::perspective(
        //     3.1415926535 / 2.0,
        //     aspect_ratio,
        //     0.01, 
        //     100.0, 
        // );

        let height = 1.0;
        let width = height * aspect_ratio;
        let projection = glam::camera::rh::proj::directx::orthographic(
            0.0,
            width,
            0.0,
            height,
            -1.0,
            1.0
        );

        return projection * translation;
    }
}