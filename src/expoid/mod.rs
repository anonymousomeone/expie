use wgpu::Queue;

mod expie;
mod format;

pub trait Renderable {
    fn render(queue: Queue);
    fn load_textures(queue: Queue);
    fn drop_textures(queue: Queue);
}

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