use super::{copy_buffer, CommandPool, Device};
use ash::{version::DeviceV1_0, vk};
pub struct IndexBuffer {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
    pub buffer_size: u64,
}
impl IndexBuffer {
    pub fn new(device: &mut Device, command_pool: &mut CommandPool, indicies: Vec<u32>) -> Self {
        let buffer_size = indicies.len() * std::mem::size_of::<f32>();
        let (staging_buffer, staging_memory) = device.create_buffer(
            buffer_size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::SharingMode::EXCLUSIVE,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        unsafe {
            let memory_ptr = device
                .device
                .map_memory(
                    staging_memory,
                    0,
                    buffer_size as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory");
            std::ptr::copy_nonoverlapping(
                indicies.as_ptr() as *const std::ffi::c_void,
                memory_ptr,
                buffer_size,
            );
            device.device.unmap_memory(staging_memory);
        }
        let (buffer, buffer_memory) = device.create_buffer(
            buffer_size as u64,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        copy_buffer(
            device,
            command_pool,
            &staging_buffer,
            &buffer,
            buffer_size as u64,
        );
        unsafe {
            device.device.destroy_buffer(staging_buffer, None);
            device.device.free_memory(staging_memory, None);
        }
        Self {
            buffer,
            buffer_memory,
            buffer_size: buffer_size as u64,
        }
    }
}
