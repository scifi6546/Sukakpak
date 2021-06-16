use super::{Core, DepthBuffer, GraphicsPipeline, PresentImage};
use ash::{version::DeviceV1_0, vk};
pub struct Framebuffer {
    pub framebuffers: Vec<vk::Framebuffer>,
}
impl Framebuffer {
    pub fn new(
        core: &mut Core,
        present_images: &mut PresentImage,
        pipeline: &mut GraphicsPipeline,
        depth_buffer: &DepthBuffer,
        width: u32,
        height: u32,
    ) -> Self {
        let framebuffers: Vec<vk::Framebuffer> = present_images
            .present_image_views
            .iter()
            .map(|image_view| {
                let attachments = [*image_view, depth_buffer.view];
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(pipeline.clear_pipeline.renderpass)
                    .attachments(&attachments)
                    .width(width)
                    .height(height)
                    .layers(1);
                unsafe {
                    core.device
                        .create_framebuffer(&create_info, None)
                        .expect("failed to create_framebuffer")
                }
            })
            .collect();

        Self { framebuffers }
    }
    pub fn free(&mut self, core: &mut Core) {
        unsafe {
            for framebuffer in self.framebuffers.iter() {
                core.device.destroy_framebuffer(*framebuffer, None);
            }
        }
    }
}
