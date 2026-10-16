use std::{borrow::Cow, sync::Arc};

use rapier2d::pipeline::{DebugRenderMode, DebugRenderPipeline, DebugRenderStyle, PhysicsWorld};
use wgpu::{RenderPipeline, util::DeviceExt};
use winit::{
    event_loop::OwnedDisplayHandle,
    window::Window,
};

use crate::{State, graphics::{LineCollector, Point, camera::Camera}};

pub struct Renderer {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    render_pipeline: RenderPipeline,

    camera: Camera,

    debug_renderer_backend: LineCollector,
    debug_renderer: DebugRenderPipeline,
}

impl Renderer {
    pub async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> Renderer {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();
        // size.height = size.height - 1;

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });
        
        let camera = Camera::new(&device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&camera.bind_group_layout)
            ],
            immediate_size: 0,
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

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
                targets: &[Some(swapchain_format.into())],
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

        let state = Renderer {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            render_pipeline,

            camera,

            debug_renderer,
            debug_renderer_backend
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Inherit,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    pub fn render(&mut self, world: &PhysicsWorld) {
        // Create texture view.
        // NOTE: We must handle Timeout because the surface may be unavailable
        // (e.g., when the window is occluded on macOS).
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                return;
            }
        };
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

            let mut encoder = self.device.create_command_encoder(&Default::default());

        // Create the renderpass which will clear the screen.
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // If you wanted to call any drawing commands, they would go here.
        renderpass.set_viewport(
            0.0, 
            0.0, 
            self.size.width as f32, 
            (self.size.height - 1) as f32, 
            0.0, 
            1.0
        );
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

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(surface_texture);
    }
}
