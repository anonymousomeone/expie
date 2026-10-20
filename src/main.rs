mod graphics;
mod expoid;
mod editor;

use crate::{editor::editor::Editor, expoid::container::Container, graphics::{Renderable, Renderer}};

use std::sync::Arc;

use egui::FullOutput;
use image::GenericImageView;
use rapier2d::prelude::*;
use wgpu::{BindGroupLayoutEntry, BindingType, ShaderStages, TextureSampleType, TextureViewDimension};
use winit::{
    application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize}, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Icon, WindowAttributes, WindowId, WindowLevel},
};

struct App {
    renderer: Option<Renderer>,
    state: State,
}

pub struct State {
    physics_world: PhysicsWorld,
    entities: Vec<Box<dyn Renderable>>,
    editor: Editor,
}

impl App {
    pub fn new() -> App {
        let mut physics_world = PhysicsWorld::new();

        let rigid_body = RigidBodyBuilder::new(RigidBodyType::Fixed)
            .pose(Pose::new(Vector::new(0.0, 0.0), 0.0))
            .can_sleep(true)
            .ccd_enabled(false)
            .build();

        // let shape = ColliderBuilder::voxels_from_points(
        //     Vector::new(0.1, 0.1),
        //     &[
        //         Vector::new(0.0, 0.0),
        //     ],
        // );

        let shape = ColliderBuilder::cuboid(0.4, 0.1);

        let rigid_body_handle = physics_world.insert_body(rigid_body);
        physics_world.insert_collider(shape, Some(rigid_body_handle));

        let editor = Editor::new();
        
        let state = State {
            physics_world,
            entities: Vec::new(),
            editor,
        };

        App {
            renderer: None,
            state,
        }
    }

    fn ui(raw_input: egui::RawInput, ctx: &egui::Context, state: &mut State) -> FullOutput {
        ctx.run_ui(raw_input, |ui| {
            ui.heading("hello wold");
            state.editor.ui(ctx, ui);
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let image = image::load_from_memory(include_bytes!("../assets/logo.png")).unwrap();
        let (width, height) = image.dimensions();

        let icon = Icon::from_rgba(image.into_bytes(), width, height).unwrap();
        let attributes = WindowAttributes::default()
            // .with_transparent(true)
            // .with_decorations(false)
            .with_title("Expie")
            .with_window_icon(Some(icon))
            // .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .with_inner_size(PhysicalSize::new(1920, 1081))
            .with_position(PhysicalPosition::new(0, 0));
            // .with_window_level(WindowLevel::AlwaysOnTop);

        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .unwrap(),
        );

        // window.set_cursor_hittest(false).unwrap();

        let mut renderer = pollster::block_on(Renderer::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));

        // Builder for a body with a status specified by an enum.
        let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .pose(Pose::new(Vector::new(0.05, 2.0), 0.4))
            // .linvel(Vector::new(1.0, 2.0))
            .angvel(2.0)
            .can_sleep(false)
            .ccd_enabled(false)
            .sleeping(false)
            .build();

        let collider = ColliderBuilder::new(SharedShape::ball(0.1))
            .density(800.0)
            .friction(1.0)
            // .restitution(0.9)
            // .sensor(true)
            .build();

        let rigid_body_handle = self.state.physics_world.insert_body(rigid_body);
        self.state.physics_world.insert_collider(collider, Some(rigid_body_handle));

        let resources = &mut renderer.resource_manager;
        let crate_id = resources.load_texture_from_path("./assets/objects/textures/Containercrate.png");
        let sampler = resources.new_sampler();

        let layout_entries = vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
        let layout = resources.new_layout(layout_entries, None);

        let texture_view = resources.get_texture(crate_id).create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = resources.get_sampler(sampler);

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }
        ];
        let bind_group = resources.new_bind_group(layout, entries, None);

        let container_crate = Container::new(
            &mut self.state.physics_world,
            Vec2::new(0.5, 0.0),
            Vec2::new(0.2, 0.2),
            bind_group,
            expoid::container::ContainerVariant::Container
        );

        self.state.entities.push(Box::new(container_crate));
        

        window.request_redraw();
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = self.renderer.as_mut().unwrap();
        let window = renderer.get_window();

        let response = renderer.ui_state.on_window_event(&window, &event);
        if response.repaint {
            window.request_redraw();
        }
        if response.consumed {
            if !matches!(event, WindowEvent::RedrawRequested) {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.state.physics_world.step();
                
                let ctx = &renderer.ui_context;
                let raw_input = renderer.ui_state.take_egui_input(&renderer.get_window());
                
                let output = App::ui(raw_input, ctx, &mut self.state);
                renderer.render(&mut self.state, output);

                // Emits a new redraw requested event.
                renderer.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                renderer.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}