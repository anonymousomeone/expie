use crate::{State, graphics::resource::ResourceManager};

pub mod debug_pass;
pub mod main_pass;

pub trait Pass {
    fn render(
        &mut self,
        resource_manager: &ResourceManager,
        renderpass: &mut wgpu::RenderPass,
        state: &mut State
    );
}