use super::{find_memorytype_index, Device};
use ash::{version::DeviceV1_0, vk};
pub struct UniformBuffer<const SIZE: usize> {
    pub layout: [vk::DescriptorSetLayout; 1],
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
}
impl<const SIZE: usize> UniformBuffer<SIZE> {
    pub fn new(device: &mut Device) -> Self {
        let layout_binding = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .build()];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_binding);
        let layout = unsafe {
            [device
                .device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create layout")]
        };
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(SIZE as u64)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe {
            device
                .device
                .create_buffer(&buffer_create_info, None)
                .expect("failed to create buffer")
        };
        let buffer_memory_requirements =
            unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(buffer_memory_requirements.size)
            .memory_type_index(
                find_memorytype_index(
                    &buffer_memory_requirements,
                    &device.memory_properties,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
                .expect("failed to find memory type"),
            );
        let buffer_memory = unsafe {
            device
                .device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allocate buffer memory")
        };
        unsafe {
            device
                .device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("failed to bind buffer");
        }
        Self {
            layout,
            buffer,
            buffer_memory,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            device.device.destroy_buffer(self.buffer, None);
            for layout in self.layout.iter() {
                device.device.destroy_descriptor_set_layout(*layout, None);
            }
        }
    }
}
