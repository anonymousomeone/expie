use std::borrow::Cow;

use rapier2d::pipeline::{DebugRenderMode, DebugRenderPipeline, DebugRenderStyle};
use wgpu::util::DeviceExt;

use crate::State;
use crate::graphics::resource::ResourceManager;
use crate::graphics::{LineCollector, Point};
use crate::graphics::camera::Camera;
use crate::graphics::passes::Pass;

pub struct DebugPass {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,

    debug_renderer_backend: LineCollector,
    debug_renderer: DebugRenderPipeline,
}

impl DebugPass {
    pub fn new(
        device: wgpu::Device, 
        queue: wgpu::Queue,
        target: wgpu::ColorTargetState,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> DebugPass {
        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("line_shader.wgsl"))),
        });
        
        let camera = Camera::new(&device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&camera.bind_group_layout)
            ],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(Point::desc())
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(target.into())],
            }),
            // primitive: wgpu::PrimitiveState::default(),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList, // or LineStrip
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Often disabled for lines
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let debug_renderer = DebugRenderPipeline::new(
            DebugRenderStyle::default(),
            DebugRenderMode::COLLIDER_SHAPES | DebugRenderMode::CONTACTS,
        );
        let debug_renderer_backend = LineCollector { lines: vec![] };

        DebugPass { 
            device,
            queue,
            render_pipeline,
            camera,
            size,

            debug_renderer,
            debug_renderer_backend
        }

    }
}

impl Pass for DebugPass {
    fn render(
        &mut self,
        _resource_manager: &ResourceManager,
        renderpass: &mut wgpu::RenderPass,
        state: &mut State
    ) {
        let world = &state.physics_world;
        renderpass.set_pipeline(&self.render_pipeline);
        // renderpass.draw(0..3, 0..1);

        self.debug_renderer_backend.lines.clear();
        self.debug_renderer.render(
            &mut self.debug_renderer_backend,
            &world.bodies,
            &world.colliders,
            &world.impulse_joints,
            &world.multibody_joints,
            &world.narrow_phase,
            &world.soft_bodies
        );

        // println!("{:#?}", self.debug_renderer_backend.lines);

        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.debug_renderer_backend.lines),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let aspect_ratio: f32 = self.size.width as f32 / self.size.height as f32;
        let matrix = self.camera.to_matrix(aspect_ratio).to_cols_array_2d();
        self.queue.write_buffer(
            &self.camera.buffer, 
            0, 
            bytemuck::bytes_of(&matrix)
        );

        renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        renderpass.set_bind_group(0, Some(&self.camera.bind_group), &[]);
        renderpass.draw(0..self.debug_renderer_backend.lines.len() as u32, 0..1);
    }
}