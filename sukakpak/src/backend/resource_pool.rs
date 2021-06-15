use super::{Core, VertexLayout};
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use gpu_allocator::{
    AllocationCreateDesc, MemoryLocation, SubAllocation, VulkanAllocator, VulkanAllocatorCreateDesc,
};
use std::mem::{size_of, ManuallyDrop};
pub struct VertexBufferPool {
    allocator: ManuallyDrop<VulkanAllocator>,
}
impl VertexBufferPool {
    pub fn new(core: &Core) -> Self {
        Self {
            allocator: ManuallyDrop::new(VulkanAllocator::new(&VulkanAllocatorCreateDesc {
                instance: core.instance.clone(),
                device: core.device.clone(),
                physical_device: core.physical_device,
                debug_settings: Default::default(),
            })),
        }
    }
    pub fn allocate_buffer(
        &mut self,
        core: &mut Core,
        mesh: Vec<u8>,
        layout: VertexLayout,
    ) -> Result<VertexBufferAllocation> {
        let (binding_description, input_description) = match layout {
            VertexLayout::XYZ_F32 => (
                *vk::VertexInputBindingDescription::builder()
                    .binding(0)
                    .stride(3 * size_of::<f32>() as u32)
                    .input_rate(vk::VertexInputRate::VERTEX),
                vec![*vk::VertexInputAttributeDescription::builder()
                    .binding(0)
                    .location(0)
                    .format(vk::Format::R32G32B32_SFLOAT)],
            ),
        };
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(mesh.len() as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { core.device.create_buffer(&buffer_create_info, None)? };
        let requirements = unsafe { core.device.get_buffer_memory_requirements(buffer) };
        let allocation = self.allocator.allocate(&AllocationCreateDesc {
            name: "vertex buffer",
            requirements,
            location: MemoryLocation::CpuToGpu,
            linear: true,
        })?;
        unsafe {
            core.device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
            let mesh_len = mesh.len();
            std::ptr::copy_nonoverlapping(
                mesh.as_ptr() as *const std::ffi::c_void,
                allocation
                    .mapped_ptr()
                    .expect("failed to map mesh ptr")
                    .as_ptr(),
                mesh_len,
            );
        }
        Ok(VertexBufferAllocation {
            allocation,
            buffer,
            binding_description,
            input_description,
        })
    }
    pub fn free(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
        }
    }
}
pub struct VertexBufferAllocation {
    allocation: SubAllocation,
    buffer: vk::Buffer,
    binding_description: vk::VertexInputBindingDescription,
    input_description: Vec<vk::VertexInputAttributeDescription>,
}
impl VertexBufferAllocation {
    pub fn free(mut self, core: &mut Core, vertex_pool: &mut VertexBufferPool) {
        vertex_pool.allocator.free(self.allocation);
        unsafe {
            core.device.destroy_buffer(self.buffer, None);
        }
    }
}
