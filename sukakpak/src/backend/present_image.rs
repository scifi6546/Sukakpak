use super::Core;
use ash::version::DeviceV1_0;
use ash::vk;
pub struct PresentImage {
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,
}
impl PresentImage {
    /// Gets Number of swapchain images in present images.Backend
    #[allow(dead_code)]
    pub fn num_swapchain_images(&self) -> usize {
        self.present_images.len()
    }
    pub fn new(core: &mut Core) -> Self {
        let present_images = unsafe { core.swapchain_loader.get_swapchain_images(core.swapchain) }
            .expect("failed to get swapchain devices");
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
        Self {
            present_images,
            present_image_views,
        }
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
