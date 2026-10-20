use std::sync::Arc;

use winit::{
    event_loop::OwnedDisplayHandle,
    window::Window,
};

use egui_wgpu::{Renderer as UiRenderer, RendererOptions};

use crate::{State, graphics::{camera::Camera, passes::{Pass, debug_pass::DebugPass, main_pass::MainPass}, resource::ResourceManager}};

pub struct Renderer {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    pub camera: Camera,
    pub resource_manager: ResourceManager,

    pub ui_context: egui::Context,
    pub ui_state: egui_winit::State,
    pub ui_renderer: UiRenderer,

    passes: Vec<Box<dyn Pass>>,
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
            .request_device(&wgpu::DeviceDescriptor {
                ..Default::default()
            })
            .await
            .unwrap();

        let size = window.inner_size();
        // size.height = size.height - 1;

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let camera = Camera::new(&device, size);

        let resource_manager = ResourceManager::new(device.clone(), queue.clone());

        let mut passes: Vec<Box<dyn Pass>> = Vec::new();

        let main_pass = MainPass::new(
            device.clone(),
            queue.clone(),
            swapchain_format.into(),
            &camera.bind_group_layout
        );

        let debug_pass = DebugPass::new(
            device.clone(),
            queue.clone(),
            swapchain_format.into(),
            &camera.bind_group_layout
        );

        passes.push(Box::new(main_pass));
        passes.push(Box::new(debug_pass));

        
        let ui_context = egui::Context::default();

        let ui_state = egui_winit::State::new(
            ui_context.clone(),
            ui_context.viewport_id(),
            &window,
            Some(window.scale_factor() as f32),
            None,
            None
        );

        let ui_renderer = UiRenderer::new(
            &device,
            surface_format,
            RendererOptions {
                msaa_samples: 0,
                depth_stencil_format: None,
                dithering: true,
                predictable_texture_filtering: false,
            },
        );

        let state = Renderer {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            camera,
            resource_manager,

            ui_context,
            ui_state,
            ui_renderer,

            passes,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone()
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
        self.camera.viewport_size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    pub fn render(&mut self, state: &mut State, full_output: egui::FullOutput) {
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

        let tris = self
            .ui_context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, deltas) in &full_output.textures_delta.set {
            for delta in deltas {
                self.ui_renderer
                    .update_texture(&self.device, &self.queue, *id, delta);
            }
        }

        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let user_cmd_bufs = self.ui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen
        );

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
        }).forget_lifetime();

        // If you wanted to call any drawing commands, they would go here.
        renderpass.set_viewport(
            0.0, 
            0.0, 
            self.size.width as f32, 
            (self.size.height - 1) as f32, 
            0.0, 
            1.0
        );

        for pass in &mut self.passes {
            pass.render(
                &self.resource_manager,
                &mut renderpass, 
                state,
                &self.camera
            );
        }

        self.ui_renderer.render(&mut renderpass, &tris, &screen);

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit(user_cmd_bufs.into_iter().chain([encoder.finish()]));
        self.window.pre_present_notify();
        self.queue.present(surface_texture);

        for id in &full_output.textures_delta.free {
            self.ui_renderer.free_texture(id);
        }
    }

}
