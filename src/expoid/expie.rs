use rapier2d::prelude::*;

use crate::State;
use crate::expoid::Transform;
use crate::graphics::Renderable;

pub struct Expie {
    pub limbs: Vec<Limb>,

    pub rigidbody: RigidBody
}

impl Expie {
    pub fn new(physics_world: PhysicsWorld) -> Expie {
        let rigidbody = RigidBodyBuilder::kinematic_position_based()
            .build();
        Expie {
            limbs: Vec::new(),
            rigidbody,
        }
    }
}

impl Renderable for Expie {
    fn render(&self,
        queue: &wgpu::Queue,
        state: &State,
        resources: &crate::graphics::resource::ResourceManager,
        pass: &mut wgpu::RenderPass,
        model_buffer: &wgpu::Buffer
    ) {
        todo!()
    }

    fn load_textures(&self, queue: wgpu::Queue) {
        todo!()
    }

    fn drop_textures(&self, queue: wgpu::Queue) {
        todo!()
    }
}

pub struct Limb {
    pub transform: Transform
}