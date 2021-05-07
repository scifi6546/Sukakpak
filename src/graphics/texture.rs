use super::{find_memorytype_index, CommandQueue, Device};
use ash::{version::DeviceV1_0, vk};
use image::io::Reader as ImageReader;
pub struct Texture {
    image_data: image::RgbaImage,
    transfer_memory: vk::DeviceMemory,
    buffer: vk::Buffer,
    image: vk::Image,
    image_memory: vk::DeviceMemory,
}
impl Texture {
    pub fn new(device: &mut Device, command_queue: &mut CommandQueue) -> Self {
        let image_data = ImageReader::open("screenshot.png")
            .expect("failed to load image")
            .decode()
            .expect("failed to decode image")
            .to_rgba8();

        let (buffer, transfer_memory) = device.create_buffer(
            image_data.as_raw().len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::SharingMode::EXCLUSIVE,
        );
        unsafe {
            let memory_ptr = device
                .device
                .map_memory(
                    transfer_memory,
                    0,
                    image_data.as_raw().len() as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory");
            std::ptr::copy_nonoverlapping(
                image_data.as_raw().as_ptr() as *const std::ffi::c_void,
                memory_ptr,
                image_data.as_raw().len(),
            );
            device.device.unmap_memory(transfer_memory);
        }

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(
                vk::Extent3D::builder()
                    .width(image_data.width())
                    .height(image_data.height())
                    .depth(1)
                    .build(),
            )
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);
        let image = unsafe { device.device.create_image(&image_create_info, None) }
            .expect("failed to create image");
        let memory_reqirements = unsafe { device.device.get_image_memory_requirements(image) };
        let memory_alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_reqirements.size)
            .memory_type_index(
                find_memorytype_index(
                    &memory_reqirements,
                    &device.memory_properties,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )
                .expect("failed to find memory"),
            );
        let image_memory = unsafe { device.device.allocate_memory(&memory_alloc_info, None) }
            .expect("failed to allocate device memory");
        unsafe {
            device
                .device
                .bind_image_memory(image, image_memory, 0)
                .expect("failed to bind memory");
        }
        let command_buffer = unsafe { command_queue.create_onetime_buffer(device) };
        Self {
            image,
            image_data,
            buffer,
            transfer_memory,
            image_memory,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            device.device.free_memory(self.image_memory, None);
            device.device.destroy_image(self.image, None);
            device.device.free_memory(self.transfer_memory, None);
            device.device.destroy_buffer(self.buffer, None);
        }
    }
}
