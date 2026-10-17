use crate::State;

pub mod debug_pass;
pub mod main_pass;

pub trait Pass {
    fn render(&mut self, renderpass: &mut wgpu::RenderPass, state: &mut State);
}