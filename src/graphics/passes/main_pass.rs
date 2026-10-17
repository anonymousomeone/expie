use std::borrow::Cow;

use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::State;
use crate::graphics::{Renderer, Vertex, util};
use crate::graphics::camera::Camera;
use crate::graphics::passes::Pass;

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] },

    Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 0.0] },
];

pub struct MainPass {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,

    vertex_buffer: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup
}

impl MainPass {
    pub fn new(
        device: wgpu::Device, 
        queue: wgpu::Queue,
        target: wgpu::ColorTargetState,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> MainPass {
        let diffuse_bytes = include_bytes!("../../../assets/objects/textures/Containercrate.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        let dimensions = diffuse_image.dimensions();

        let texture = util::load_texture(
            &device,
            &queue,
            dimensions.0,
            dimensions.1,
            &diffuse_rgba
        );

        let diffuse_texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

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

        let texture_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("main_shader.wgsl"))),
        });        
        
        let camera = Camera::new(&device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&camera.bind_group_layout),
                Some(&texture_bind_group_layout),
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

        MainPass { 
            device,
            queue,
            render_pipeline,
            camera,
            size,

            vertex_buffer,
            texture_bind_group,
        }

    }
}

impl Pass for MainPass {
    fn render(&mut self, renderpass: &mut wgpu::RenderPass, state: &mut State) {
        renderpass.set_pipeline(&self.render_pipeline);
        // renderpass.draw(0..3, 0..1);
        
        // println!("{:#?}", self.debug_renderer_backend.lines);
        
        let aspect_ratio: f32 = self.size.width as f32 / self.size.height as f32;
        let matrix = self.camera.to_matrix(aspect_ratio).to_cols_array_2d();
        self.queue.write_buffer(
            &self.camera.buffer, 
            0, 
            bytemuck::bytes_of(&matrix)
        );
        
        renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        renderpass.set_bind_group(0, Some(&self.camera.bind_group), &[]);
        renderpass.set_bind_group(1, &self.texture_bind_group, &[]);
        renderpass.draw(0..VERTICES.len() as u32, 0..1);
    }
}