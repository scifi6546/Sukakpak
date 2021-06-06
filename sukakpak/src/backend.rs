use anyhow::Result;

use nalgebra::Vector2;
mod render_core;
use render_core::Core;
pub struct BackendCreateInfo {
    pub default_size: Vector2<u32>,
    pub name: String,
}
pub struct Backend {
    window: winit::window::Window,
    core: Core,
}
impl Backend {
    pub fn new(create_info: BackendCreateInfo) -> Result<Self> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title(create_info.name.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                create_info.default_size.x,
                create_info.default_size.y,
            ))
            .build(&event_loop)?;
        let core = Core::new(&window, &create_info)?;
        Ok(Self { window, core })
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        unsafe {
            self.core.free();
        }
    }
}
