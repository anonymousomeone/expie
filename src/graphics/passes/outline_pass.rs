use std::borrow::Cow;

use glam::Mat4;
use wgpu::util::DeviceExt;

use crate::State;
use crate::graphics::resource::ResourceManager;
use crate::graphics::{Outline, Vertex};
use crate::graphics::camera::RenderCamera;
use crate::graphics::passes::Pass;

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] },

    Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 0.0] },
];

pub struct OutlinePass {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    model_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
    outline_buffer: wgpu::Buffer,
    outline_bind_group: wgpu::BindGroup,
}

impl OutlinePass {
    pub fn new(
        device: wgpu::Device, 
        queue: wgpu::Queue,
        target: wgpu::ColorTargetState,
        camera_bind_group: &wgpu::BindGroupLayout
    ) -> OutlinePass {
        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("model_bind_group_layout"),
        });

        let model_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Model Buffer"),
                contents: bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let model_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
                layout: &model_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_buffer.as_entire_binding(),
                    },
                ],
                label: Some("model_bind_group"),
            }
        );

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let outline_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
            ],
            label: Some("outline_bind_group_layout"),
        });

        let outline_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Outline Buffer"),
                contents: bytemuck::cast_slice(&[Outline::new([0.0; 4], 0.0)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let outline_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
                layout: &outline_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: outline_buffer.as_entire_binding(),
                    },
                ],
                label: Some("outline_bind_group"),
            }
        );

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("outline_shader.wgsl"))),
        });        
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(camera_bind_group),
                Some(&model_bind_group_layout),
                Some(&texture_bind_group_layout),
                Some(&outline_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("main"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(Vertex::desc())
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(target.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        OutlinePass { 
            device,
            queue,
            render_pipeline,

            vertex_buffer,
            model_buffer,
            model_bind_group,
            outline_buffer,
            outline_bind_group,
        }

    }
}

impl Pass for OutlinePass {
    fn render(
        &mut self,
        resource_manager: &ResourceManager,
        renderpass: &mut wgpu::RenderPass,
        state: &mut State,
        camera: &RenderCamera
    ) {
        renderpass.set_pipeline(&self.render_pipeline);

        // renderpass.draw(0..3, 0..1);
        
        // println!("{:#?}", self.debug_renderer_backend.lines);
        
        let aspect_ratio: f32 = camera.viewport_size.width as f32 / camera.viewport_size.height as f32;
        let matrix = camera.to_matrix(aspect_ratio).to_cols_array_2d();
        self.queue.write_buffer(
            &camera.buffer, 
            0, 
            bytemuck::bytes_of(&matrix)
        );

        renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        renderpass.set_bind_group(0, Some(&camera.bind_group), &[]);
        renderpass.set_bind_group(1, Some(&self.model_bind_group), &[]);
        renderpass.set_bind_group(3, Some(&self.outline_bind_group), &[]);
        
        for entity in &mut state.entities {
            if !entity.is_outlined() { continue }

            self.queue.write_buffer(
                &self.outline_buffer, 
                0, 
                bytemuck::bytes_of(&entity.outline_style())
            );
            
            entity.render(
                &self.queue,
                &state.physics_world,
                resource_manager,
                renderpass,
                &self.model_buffer
            );
        }
    }
}