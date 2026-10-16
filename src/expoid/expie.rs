use rapier2d::prelude::*;

use crate::expoid::{Renderable, Transform};

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
    fn render(queue: wgpu::Queue) {
        todo!()
    }

    fn load_textures(queue: wgpu::Queue) {
        todo!()
    }

    fn drop_textures(queue: wgpu::Queue) {
        todo!()
    }
}

pub struct Limb {
    pub transform: Transform
}