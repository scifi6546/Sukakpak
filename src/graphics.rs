mod commands;
mod device;
mod framebuffer;
mod present_images;
mod shaders;
use commands::CommandQueue;
pub use device::Device;
use framebuffer::Framebuffer;
use present_images::PresentImage;
use shaders::GraphicsPipeline;
pub struct Context {
    device: Device,
    present_images: PresentImage,
    graphics_pipeline: GraphicsPipeline,
    framebuffer: Framebuffer,
    command_queue: CommandQueue,
    width: u32,
    height: u32,
    event_loop: &'static winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
}
impl Context {
    pub fn new(
        title: &str,
        event_loop: &'static winit::event_loop::EventLoop<()>,
        width: u32,
        height: u32,
    ) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(&event_loop)
            .unwrap();
        let mut device = Device::new(&window, width, height);
        let mut present_images = PresentImage::new(&mut device);
        let mut graphics_pipeline = GraphicsPipeline::new(&mut device, width, height);
        let mut framebuffer = Framebuffer::new(
            &mut device,
            &mut present_images,
            &mut graphics_pipeline,
            width,
            height,
        );
        let command_queue = CommandQueue::new(
            &mut device,
            &mut graphics_pipeline,
            &mut framebuffer,
            width,
            height,
        );
        Self {
            device,
            present_images,
            graphics_pipeline,
            framebuffer,
            command_queue,
            width,
            height,
            event_loop,
            window,
        }
    }
    pub fn start_event_loop(&mut self) {}
}
impl Drop for Context {
    fn drop(&mut self) {
        self.command_queue.free(&mut self.device);
        self.framebuffer.free(&mut self.device);
        self.graphics_pipeline.free(&mut self.device);
        self.present_images.free(&mut self.device);
        self.device.free();
    }
}
