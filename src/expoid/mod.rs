mod expie;
mod format;
pub mod container;

pub struct Transform {
    x: f32,
    y: f32,
    /// rotation
    r: f32,
}

impl Transform {
    pub fn new(x: f32, y: f32, r: f32) -> Transform {
        Transform { x, y, r }
    }
}