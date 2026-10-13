mod graphics;
mod expoid;

use crate::graphics::GraphicsState;

use std::sync::Arc;

use rapier2d::prelude::*;
use winit::{
    application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize}, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Fullscreen, WindowAttributes, WindowId, WindowLevel},
};

struct App {
    graphics_state: Option<GraphicsState>,
    physics_world: PhysicsWorld
}

impl App {
    pub fn new() -> App {
        let mut physics_world = PhysicsWorld::new();

        let shape = ColliderBuilder::voxels_from_points(
            Vector::new(1.0, 1.0),
            &[
                Vector::new(0.0, 0.0),
                Vector::new(1.0, 1.0),
                Vector::new(-1.0, 1.0),
            ],
        );

        physics_world.insert_collider(shape, None);

        App {
            graphics_state: None,
            physics_world,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_transparent(true)
            .with_decorations(false)
            // .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .with_inner_size(PhysicalSize::new(1920, 1081))
            .with_position(PhysicalPosition::new(0, 0))
            .with_window_level(WindowLevel::AlwaysOnTop);

        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .unwrap(),
        );

        window.set_cursor_hittest(false).unwrap();

        let state = pollster::block_on(GraphicsState::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        self.graphics_state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.graphics_state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
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