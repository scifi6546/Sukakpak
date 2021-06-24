use super::{CommandPool, Core, ResourcePool, TextureAllocation};
use anyhow::Result;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use gpu_allocator::SubAllocation;
use nalgebra::Vector2;
pub struct ColorBuffer {
    pub present_images: Vec<(vk::Image, Option<SubAllocation>)>,
    pub present_image_views: Vec<vk::ImageView>,
}
#[derive(Clone, Copy)]
pub enum AttachmentType {
    Swapchain,
    UserFramebuffer,
}
impl ColorBuffer {
    /// Gets Number of swapchain images in present images.Backend
    #[allow(dead_code)]
    pub fn num_swapchain_images(&self) -> usize {
        self.present_images.len()
    }
    //builds new color buffer, dimensions are ignored if the attachment is for the swapchain
    pub fn new(
        core: &mut Core,
        command_pool: &mut CommandPool,
        resource_pool: &mut ResourcePool,
        attachment_type: AttachmentType,
        dimensions: Option<Vector2<u32>>,
    ) -> Result<Self> {
        let present_images: Vec<(vk::Image, Option<SubAllocation>)> = match attachment_type {
            AttachmentType::Swapchain => {
                unsafe { core.swapchain_loader.get_swapchain_images(core.swapchain)? }
                    .iter()
                    .map(|image| (*image, None))
                    .collect()
            }
            AttachmentType::UserFramebuffer => {
                let len =
                    unsafe { core.swapchain_loader.get_swapchain_images(core.swapchain) }?.len();
                (0..len)
                    .map(|_| {
                        let (image, suballoc) = resource_pool
                            .new_image(
                                core,
                                core.surface_format.format,
                                vk::ImageUsageFlags::COLOR_ATTACHMENT
                                    | vk::ImageUsageFlags::SAMPLED,
                                dimensions.expect("needs dimensions"),
                            )
                            .expect("failed to allocate image");
                        TextureAllocation::transition_image_layout(
                            core,
                            command_pool,
                            &image,
                            vk::ImageAspectFlags::COLOR,
                            vk::ImageLayout::UNDEFINED,
                            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        );
                        (image, Some(suballoc))
                    })
                    .collect()
            }
        };
        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|(image, _suballoc)| {
                let create_image_view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(core.surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(*image);
                unsafe { core.device.create_image_view(&create_image_view_info, None) }
                    .expect("failed to create image")
            })
            .collect();
        Ok(Self {
            present_images,
            present_image_views,
        })
    }
    /// clears resources, warning once called object is in invalid state
    pub fn free(&mut self, core: &mut Core) {
        unsafe {
            core.device.device_wait_idle().expect("failed to wait");
            for view in self.present_image_views.iter() {
                core.device.destroy_image_view(*view, None);
            }
        }
    }
}
