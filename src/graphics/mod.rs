pub mod graphics;
pub mod camera;
pub mod passes;
pub mod resource;
pub mod sprite;
mod util;

use bytemuck::{Pod, Zeroable};
pub use graphics::Renderer;
use rapier2d::{math::Vector, pipeline::{DebugColor, DebugRenderBackend, DebugRenderObject, PhysicsWorld}};

use wgpu::{Buffer, Queue, RenderPass};

use crate::graphics::resource::ResourceManager;

pub trait Renderable {
    fn render(&mut self,
        queue: &Queue,
        physics_world: &PhysicsWorld,
        resources: &ResourceManager,
        pass: &mut RenderPass,
        model_buffer: &Buffer
    );

    fn set_outline(&mut self, outlined: bool);

    fn is_outlined(&self) -> bool {
        false
    }

    fn outline_style(&self) -> Outline {
        Outline::new([1.0, 1.0, 0.0, 1.0], 2.0)
    }
}

pub struct LineCollector {
    pub lines: Vec<Point>,
}

impl LineCollector {
    fn hsla_to_rgba([h, s, l, a]: [f32; 4]) -> [f32; 4] {
        let h = h.rem_euclid(360.0);
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match (h / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        [r + m, g + m, b + m, a]
    }
}

impl DebugRenderBackend for LineCollector {
    fn draw_line(&mut self, _object: DebugRenderObject, a: Vector, b: Vector, color: DebugColor) {
        let mut point = Point {
            position: a.into(),
            color: LineCollector::hsla_to_rgba(color),
        };

        self.lines.push(point);

        point.position = b.into();

        self.lines.push(point);

    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Point {
    position: [f32; 2],
    color: [f32; 4],
}

impl Point {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Point>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                }
            ]
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Outline {
    color: [f32; 4],
    thickness: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Outline {
    pub fn new(color: [f32; 4], thickness: f32) -> Outline {
        Outline {
            color,
            thickness,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}