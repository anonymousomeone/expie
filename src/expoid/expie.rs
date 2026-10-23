use rapier2d::prelude::*;

use crate::State;
use crate::graphics::Renderable;

pub struct Expie {
    pub parts: Vec<Part>,

    pub rigidbody: RigidBodyHandle
}

impl Expie {
    pub fn new(physics_world: &mut PhysicsWorld) -> Expie {
        let rigidbody = RigidBodyBuilder::kinematic_position_based()
            .build();

        let capsule = ColliderBuilder::capsule_y(0.6, 0.3);

        let (handle, _) = physics_world.insert(rigidbody, capsule);
        Expie {
            parts: Vec::new(),
            rigidbody: handle,
        }
    }
}

impl Renderable for Expie {
    fn render(&mut self,
        queue: &wgpu::Queue,
        physics_world: &PhysicsWorld,
        resources: &crate::graphics::resource::ResourceManager,
        pass: &mut wgpu::RenderPass,
        model_buffer: &wgpu::Buffer
    ) {
        todo!()
    }
    
    fn set_outline(&mut self, outlined: bool) {
        todo!()
    }
}

pub struct Part {
    // pub coll
}