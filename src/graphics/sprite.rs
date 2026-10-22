use glam::{EulerRot, Mat4, Quat, Vec2};
use rapier2d::math::Pose2;

use crate::graphics::resource::{BindGroupId, ResourceManager};

pub struct Sprite {
    bind_group: BindGroupId,
    position: Vec2,
    size: Vec2,
    rotation: f32,
}

impl Sprite {
    pub fn new(bind_group: BindGroupId) -> Sprite {
        Sprite {
            bind_group,
            position: Vec2::ZERO,
            size: Vec2::ZERO,
            rotation: 0.0,
        }
    }

    pub fn update_position(&mut self, pose: &Pose2, size: Vec2) {
        self.position = pose.translation;
        self.rotation = pose.rotation.angle();
        self.size = size;
    }

    pub fn render(
        &self,
        queue: &wgpu::Queue,
        resources: &ResourceManager,
        pass: &mut wgpu::RenderPass,
        model_buffer: &wgpu::Buffer
    ) {
        let matrix = Mat4::from_scale_rotation_translation(
            self.size.extend(0.0),
            Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, self.rotation),
            self.position.extend(0.0)
        ).to_cols_array_2d();
        queue.write_buffer(
            model_buffer, 
            0, 
            bytemuck::bytes_of(&matrix)
        );

        pass.set_bind_group(2, resources.get_bind_group(self.bind_group), &[]);
        pass.draw(0..6, 0..1);
    }
}