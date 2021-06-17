use super::{CommandPool, Core, ResourcePool, TextureAllocation};
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use gpu_allocator::SubAllocation;
use nalgebra::Vector2;
pub struct DepthBuffer {
    image: vk::Image,
    allocation: SubAllocation,
    pub view: vk::ImageView,
    depth_format: vk::Format,
}
impl DepthBuffer {
    pub fn new(
        core: &mut Core,
        command_pool: &mut CommandPool,
        resource_pool: &mut ResourcePool,
        screen_dimensions: Vector2<u32>,
    ) -> Result<Self> {
        let depth_format = core.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let (image, allocation) = resource_pool.new_image(
            core,
            depth_format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            screen_dimensions,
        )?;
        TextureAllocation::transition_image_layout(
            core,
            command_pool,
            &image,
            vk::ImageAspectFlags::DEPTH,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
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
        let view = unsafe { core.device.create_image_view(&view_info, None) }
            .expect("failed to create view");

        Ok(Self {
            image,
            allocation,
            view,
            depth_format,
        })
    }
    pub fn get_attachment(
        &self,
        load_op: vk::AttachmentLoadOp,
    ) -> (vk::AttachmentDescription, vk::AttachmentReference) {
        (
            *vk::AttachmentDescription::builder()
                .format(self.depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(load_op)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            *vk::AttachmentReference::builder()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        )
    }
    pub fn free(mut self, core: &mut Core, resource_pool: &mut ResourcePool) {
        unsafe {
            core.device.destroy_image_view(self.view, None);
            resource_pool.free_allocation(self.allocation);
            core.device.destroy_image(self.image, None);
        }
    }
}
