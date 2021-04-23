mod device;
mod present_images;
pub use device::Device;
use present_images::PresentImage;
pub struct Context {
    device: Device,
    present_images: PresentImage,
}
impl Context {
    pub fn new(window: &winit::window::Window, fallback_width: u32, fallback_height: u32) -> Self {
        let mut device = Device::new(window, fallback_width, fallback_height);
        let present_images = PresentImage::new(&mut device);
        Self {
            device,
            present_images,
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        self.present_images.free(&mut self.device);
        self.device.free();
    }
}
