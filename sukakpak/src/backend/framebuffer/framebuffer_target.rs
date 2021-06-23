use super::{ColorBuffer, Core, DepthBuffer, GraphicsPipeline};
use ash::{version::DeviceV1_0, vk};
use nalgebra::Vector2;
pub struct FrameBufferTarget {
    pub framebuffers: Vec<vk::Framebuffer>,
}
impl FrameBufferTarget {
    pub fn new(
        core: &mut Core,
        pipeline: &mut GraphicsPipeline,
        color_buffer: &ColorBuffer,
        depth_buffer: &DepthBuffer,
        resolution: Vector2<u32>,
    ) -> Self {
        let framebuffers: Vec<vk::Framebuffer> = color_buffer
            .present_image_views
            .iter()
            .map(|image_view| {
                let attachments = [*image_view, depth_buffer.view];
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(pipeline.clear_pipeline.renderpass)
                    .attachments(&attachments)
                    .width(resolution.x)
                    .height(resolution.y)
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
