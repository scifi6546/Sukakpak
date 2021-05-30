use super::{Device, Texture};
use ash::{version::DeviceV1_0, vk};
pub struct DepthBuffer {
    image: vk::Image,
    memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    depth_format: vk::Format,
}
impl DepthBuffer {
    pub fn new(device: &mut Device, width: u32, height: u32) -> Self {
        let depth_format = device.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let (image, memory) = Texture::new_image(
            device,
            depth_format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            width,
            height,
        );
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .subresource_range(
                *vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );
        let view = unsafe { device.device.create_image_view(&view_info, None) }
            .expect("failed to create view");

        Self {
            image,
            memory,
            view,
            depth_format,
        }
    }
    pub fn get_attachment(&self) -> (vk::AttachmentDescription, vk::AttachmentReference) {
        (
            *vk::AttachmentDescription::builder()
                .format(self.depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::DONT_CARE)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            *vk::AttachmentReference::builder()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        )
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            device.device.destroy_image_view(self.view, None);
            device.device.free_memory(self.memory, None);
            device.device.destroy_image(self.image, None);
        }
    }
}
