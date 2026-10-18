use crate::{State, graphics::{Renderable, resource::{BindGroupId, ResourceManager, TextureId}}};
use rapier2d::{glamx::{EulerRot, Mat4, Quat}, prelude::*};

pub struct Container {
    pub variant: ContainerVariant,
    bind_group: BindGroupId,

    rigid_body: RigidBodyHandle,
    collider: ColliderHandle,
    size: Vec2,
}

pub enum ContainerVariant {
    Medical,
    Container,
    DropCapsule,
    LifePod,
}

impl Container {
    pub fn new(
        world: &mut PhysicsWorld,
        position: Vec2,
        size: Vec2,
        bind_group: BindGroupId,
        variant: ContainerVariant,
    ) -> Container {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(position)
            .build();

        let builder = ColliderBuilder::cuboid(size.x / 2.0, size.y / 2.0);

        let collider = builder.density(3000.0)
            .friction(0.4)
            .build();

        let (body_handle, collider_handle) = world.insert(rigid_body, collider);

        Container {
            variant,
            bind_group,
            rigid_body: body_handle,
            collider: collider_handle,
            size,
        }
    }
}

impl Renderable for Container {
    fn render(&self,
        queue: &wgpu::Queue,
        state: &State,
        resources: &ResourceManager,
        pass: &mut wgpu::RenderPass,
        model_buffer: &wgpu::Buffer
    ) {
        let body = state.physics_world.bodies.get(self.rigid_body).unwrap();
        let matrix = Mat4::from_scale_rotation_translation(
            self.size.extend(0.0),
            Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
            body.translation().extend(0.0)
        ).to_cols_array_2d();
        queue.write_buffer(
            model_buffer, 
            0, 
            bytemuck::bytes_of(&matrix)
        );

        pass.set_bind_group(2, resources.get_bind_group(self.bind_group), &[]);
        pass.draw(0..6, 0..1);
    }


    fn load_textures(&self, queue: wgpu::Queue) {
        todo!()
    }

    fn drop_textures(&self, queue: wgpu::Queue) {
        todo!()
    }
    
}