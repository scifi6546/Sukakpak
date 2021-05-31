use super::{find_memorytype_index, Device, VertexBufferDesc};
use ash::{version::DeviceV1_0, vk};
use nalgebra::{Vector2, Vector3};

pub struct VertexBuffer {
    pub binding_description: [vk::VertexInputBindingDescription; 1],
    pub attributes: &'static [vk::VertexInputAttributeDescription],
    pub buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
}
#[repr(C)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub uv: Vector2<f32>,
}
impl VertexBuffer {
    pub fn new(device: &mut Device, verticies: Vec<Vertex>, desc: &VertexBufferDesc) -> Self {
        assert!(verticies.len() > 0);
        let binding_description = [desc.binding_description];
        let attributes_temp = [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(std::mem::size_of::<Vector3<f32>>() as u32)
                .build(),
        ];
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size((verticies.len() * std::mem::size_of::<Vertex>()) as u64)
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
                verticies.len() * std::mem::size_of::<Vertex>(),
            );
            device.device.unmap_memory(buffer_memory);
        }
        Self {
            binding_description,
            attributes: desc.attributes,
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
