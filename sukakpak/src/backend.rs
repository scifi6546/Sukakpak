use anyhow::Result;
use ash::vk;
use nalgebra::Vector2;
pub struct BackendCreateInfo {
    pub default_size: Vector2<u32>,
    pub name: String,
}
pub struct Backend {
    window: winit::window::Window,
}
impl Backend {
    pub fn new(create_info: BackendCreateInfo) -> Result<Self> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title(create_info.name)
            .with_inner_size(winit::dpi::LogicalSize::new(
                create_info.default_size.x,
                create_info.default_size.y,
            ))
            .build(&event_loop)?;
        Ok(Self { window })
    }
}
