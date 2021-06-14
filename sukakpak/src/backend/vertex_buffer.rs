use super::{Core, Mesh};
use ash::vk;
use gpu_allocator::{VulkanAllocator, VulkanAllocatorCreateDesc};
pub struct VertexBufferPool {
    allocator: VulkanAllocator,
}
impl VertexBufferPool {
    pub fn new(core: &Core) -> Self {
        Self {
            allocator: VulkanAllocator::new(&VulkanAllocatorCreateDesc {
                instance: core.instance.clone(),
                device: core.device.clone(),
                physical_device: core.physical_device,
                debug_settings: Default::default(),
            }),
        }
    }
    pub fn allocate_buffer(&mut self, mesh: Vec<u8>) -> VertexBufferAllocation {
        todo!()
    }
}
pub struct VertexBufferAllocation {}
