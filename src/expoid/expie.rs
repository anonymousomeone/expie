use crate::expoid::Transform;

pub struct Expie {
    pub limbs: Vec<Limb>,
}

impl Expie {
    pub fn new() -> Expie {
        Expie {
            limbs: Vec::new(),
        }
    }
}

pub struct Limb {
    pub transform: Transform
}