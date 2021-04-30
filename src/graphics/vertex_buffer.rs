use super::Device;
use ash::{version::DeviceV1_0, vk};
use nalgebra::Vector3;
struct Attribute {}
pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
pub struct VertexBuffer {
    pub binding_description: [vk::VertexInputBindingDescription; 1],
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
}

impl VertexBuffer {
    pub fn new(device: &mut Device, verticies: Vec<Vector3<f32>>) -> Self {
        let binding_description = [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vector3<f32>>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()];
        let attribute_description = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size((verticies.len() * std::mem::size_of::<Vector3<f32>>()) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
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
        unsafe {
            let memory = device
                .device
                .map_memory(
                    buffer_memory,
                    0,
                    buffer_create_info.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory");
            std::ptr::copy_nonoverlapping(
                verticies.as_ptr() as *mut std::ffi::c_void,
                memory,
                verticies.len() * std::mem::size_of::<Vector3<f32>>(),
            );
            device.device.unmap_memory(buffer_memory);
        }
        Self {
            binding_description,
            attributes: vec![attribute_description],
            buffer,
            buffer_memory,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            device.device.destroy_buffer(self.buffer, None);
            device.device.free_memory(self.buffer_memory, None);
        }
    }
}
