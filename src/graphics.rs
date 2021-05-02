mod commands;
mod device;
mod framebuffer;
mod pipeline;
mod present_images;
mod uniform;
mod vertex_buffer;
use ash::vk;
use commands::CommandQueue;
pub use device::Device;
use framebuffer::Framebuffer;
use nalgebra::Matrix4;
use nalgebra::Vector3;
use pipeline::GraphicsPipeline;
use present_images::PresentImage;
pub use uniform::UniformBuffer;
pub use vertex_buffer::VertexBuffer;
pub struct Context {
    device: Device,
    present_images: PresentImage,
    graphics_pipeline: GraphicsPipeline,
    framebuffer: Framebuffer,
    command_queue: CommandQueue,
    vertex_buffer: VertexBuffer,
    width: u32,
    height: u32,
    window: winit::window::Window,
    uniform_buffer: UniformBuffer<{ std::mem::size_of::<Matrix4<f32>>() }>,
}
impl Context {
    pub fn new(
        title: &str,
        event_loop: &winit::event_loop::EventLoop<()>,
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
        let vertex_buffer = VertexBuffer::new(
            &mut device,
            vec![
                Vector3::new(-0.5, -0.5, 0.0),
                Vector3::new(0.5, -0.5, 0.0),
                Vector3::new(0.0, 0.5, 0.0),
            ],
        );
        let uniform_buffer = UniformBuffer::new(&mut device);
        let mut graphics_pipeline =
            GraphicsPipeline::new(&mut device, &vertex_buffer, &uniform_buffer, width, height);
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
            &vertex_buffer,
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
            window,
            vertex_buffer,
            uniform_buffer,
        }
    }
    pub fn render_frame(&mut self) {
        unsafe {
            self.command_queue.render_frame(&mut self.device);
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        self.command_queue.free(&mut self.device);
        self.framebuffer.free(&mut self.device);
        self.graphics_pipeline.free(&mut self.device);
        self.uniform_buffer.free(&mut self.device);
        self.vertex_buffer.free(&mut self.device);
        self.present_images.free(&mut self.device);
        self.device.free();
    }
}
pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
