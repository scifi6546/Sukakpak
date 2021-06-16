use super::{CommandPool, Core, VertexLayout};
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use gpu_allocator::{
    AllocationCreateDesc, MemoryLocation, SubAllocation, VulkanAllocator, VulkanAllocatorCreateDesc,
};
use std::mem::{size_of, ManuallyDrop};
pub struct ResourcePool {
    allocator: ManuallyDrop<VulkanAllocator>,
}
impl ResourcePool {
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
    pub fn allocate_vertex_buffer(
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
    pub fn allocate_index_buffer(
        &mut self,
        core: &mut Core,
        command_pool: &mut CommandPool,
        indicies: Vec<u32>,
    ) -> Result<IndexBufferAllocation> {
        let buffer_size = indicies.len() * size_of::<u32>();
        let (staging_buffer, staging_memory) = self.create_buffer(
            core,
            buffer_size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::SharingMode::EXCLUSIVE,
            MemoryLocation::CpuToGpu,
        )?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                indicies.as_ptr() as *const std::ffi::c_void,
                staging_memory
                    .mapped_ptr()
                    .expect("failed to map memory")
                    .as_ptr(),
                buffer_size,
            )
        }
        let (buffer, allocation) = self.create_buffer(
            core,
            buffer_size as u64,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            MemoryLocation::GpuOnly,
        )?;
        core.copy_buffer(command_pool, &staging_buffer, &buffer, buffer_size as u64);
        unsafe { core.device.destroy_buffer(staging_buffer, None) }
        self.allocator.free(staging_memory)?;

        Ok(IndexBufferAllocation {
            buffer,
            allocation,
            buffer_size,
        })
    }
    pub fn create_buffer(
        &mut self,
        core: &mut Core,
        size: u64,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
        memory_location: MemoryLocation,
    ) -> Result<(vk::Buffer, SubAllocation)> {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(sharing_mode);
        let buffer = unsafe { core.device.create_buffer(&buffer_create_info, None)? };
        let requirements = unsafe { core.device.get_buffer_memory_requirements(buffer) };
        let allocation = self.allocator.allocate(&AllocationCreateDesc {
            name: "vertex buffer",
            requirements,
            location: memory_location,
            linear: true,
        })?;

        unsafe {
            core.device
                .bind_buffer_memory(buffer, allocation.memory(), 0)?;
        }
        Ok((buffer, allocation))
    }
    pub fn free(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
        }
    }
}
pub struct IndexBufferAllocation {
    pub buffer: vk::Buffer,
    pub allocation: SubAllocation,
    pub buffer_size: usize,
}
impl IndexBufferAllocation {
    pub fn num_indices(&self) -> usize {
        self.buffer_size / size_of::<u32>()
    }
    pub fn free(self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        resource_pool.allocator.free(self.allocation)?;
        unsafe {
            core.device.destroy_buffer(self.buffer, None);
        }
        Ok(())
    }
}
pub struct VertexBufferAllocation {
    allocation: SubAllocation,
    buffer: vk::Buffer,
    pub binding_description: vk::VertexInputBindingDescription,
    pub input_description: Vec<vk::VertexInputAttributeDescription>,
}
impl VertexBufferAllocation {
    pub fn free(mut self, core: &mut Core, resource_pool: &mut ResourcePool) {
        resource_pool.allocator.free(self.allocation);
        unsafe {
            core.device.destroy_buffer(self.buffer, None);
        }
    }
}
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
