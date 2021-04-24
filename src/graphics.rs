mod device;
mod present_images;
mod shaders;
pub use device::Device;
use present_images::PresentImage;
use shaders::GraphicsPipeline;
pub struct Context {
    device: Device,
    present_images: PresentImage,
    graphics_pipeline: GraphicsPipeline,
}
impl Context {
    pub fn new(window: &winit::window::Window, fallback_width: u32, fallback_height: u32) -> Self {
        let mut device = Device::new(window, fallback_width, fallback_height);
        let present_images = PresentImage::new(&mut device);
        let graphics_pipeline = GraphicsPipeline::new(&mut device, fallback_width, fallback_height);
        Self {
            device,
            present_images,
            graphics_pipeline,
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        self.graphics_pipeline.free(&mut self.device);
        self.present_images.free(&mut self.device);
        self.device.free();
    }
}
