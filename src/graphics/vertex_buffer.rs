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
        let buffer_memory_index = find_memorytype_index(
            &buffer_memory_requirements,
            &device.memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let alloc_info = vk::MemoryAllocateInfo::builder();
        Self {
            binding_description,
            attributes: vec![attribute_description],
            buffer,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            device.device.destroy_buffer(self.buffer, None);
        }
    }
}
