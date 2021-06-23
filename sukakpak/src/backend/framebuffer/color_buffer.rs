use super::{Core, ResourcePool};
use anyhow::Result;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
pub struct ColorBuffer {
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,
    pub sampler: vk::Sampler,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
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
    pub fn new(
        core: &mut Core,
        resource_pool: &mut ResourcePool,
        attachment_type: AttachmentType,
    ) -> Result<Self> {
        let present_images = match attachment_type {
            AttachmentType::Swapchain => unsafe {
                core.swapchain_loader.get_swapchain_images(core.swapchain)?
            },
            UserFramebuffer => todo!(),
        };
        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|&image| {
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
                    .image(image);
                unsafe { core.device.create_image_view(&create_image_view_info, None) }
                    .expect("failed to create image")
            })
            .collect();
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0)
            .max_anisotropy(
                unsafe {
                    core.instance
                        .get_physical_device_properties(core.physical_device)
                }
                .limits
                .max_sampler_anisotropy,
            );
        let sampler = unsafe { core.device.create_sampler(&sampler_info, None) }
            .expect("failed to create sampler");
        let descriptor_sets = present_images
            .iter()
            .zip(present_image_views.iter())
            .map(|(image, view)| {
                resource_pool
                    .get_texture_descriptor(core, *view, sampler)
                    .expect("failed to get descriptor")
            })
            .collect();
        Ok(Self {
            present_images,
            present_image_views,
            descriptor_sets,
            sampler,
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
