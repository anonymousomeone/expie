use crate::{graphics::{Renderable, resource::ResourceManager, sprite::Sprite}};
use rapier2d::prelude::*;

pub struct Container {
    pub variant: ContainerVariant,
    sprite: Sprite,

    rigid_body: RigidBodyHandle,
    size: Vec2,
    outlined: bool,
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
        sprite: Sprite,
        variant: ContainerVariant,
    ) -> Container {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(position)
            .gravity_scale(0.0)
            .angvel(0.2)
            .can_sleep(false)
            .build();

        let builder = ColliderBuilder::cuboid(size.x / 2.0, size.y / 2.0);

        let collider = builder.density(3000.0)
            .friction(0.4)
            .build();

        let (body_handle, _) = world.insert(rigid_body, collider);

        Container {
            variant,
            sprite,
            rigid_body: body_handle,
            size,
            outlined: false,
        }
    }
}

impl Renderable for Container {
    fn render(&mut self,
        queue: &wgpu::Queue,
        physics_world: &PhysicsWorld,
        resources: &ResourceManager,
        pass: &mut wgpu::RenderPass,
        model_buffer: &wgpu::Buffer
    ) {
        let body = physics_world.bodies.get(self.rigid_body).unwrap();
        self.sprite.update_position(body.position(), self.size);
        self.sprite.render(queue, resources, pass, model_buffer);
    }

    fn set_outline(&mut self, outlined: bool) {
        self.outlined = outlined
    }

    fn is_outlined(&self) -> bool {
        self.outlined
    }
}