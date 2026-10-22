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
}

pub struct LineCollector {
    pub lines: Vec<Point>,
}

impl DebugRenderBackend for LineCollector {
    fn draw_line(&mut self, _object: DebugRenderObject, a: Vector, b: Vector, color: DebugColor) {
        let mut point = Point {
            position: a.into(),
            color,
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