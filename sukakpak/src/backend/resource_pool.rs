use super::{CommandPool, Core, PresentImage, VertexLayout};
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use gpu_allocator::{
    AllocationCreateDesc, MemoryLocation, SubAllocation, VulkanAllocator, VulkanAllocatorCreateDesc,
};
use nalgebra::Vector2;
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
    pub fn free_allocation(&mut self, allocation: SubAllocation) -> Result<()> {
        self.allocator.free(allocation)?;
        Ok(())
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
    pub fn new_image(
        &mut self,
        core: &mut Core,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        dimensions: Vector2<u32>,
    ) -> Result<(vk::Image, SubAllocation)> {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: dimensions.x,
                height: dimensions.y,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);
        let image = unsafe { core.device.create_image(&image_create_info, None) }?;
        let requirements = unsafe { core.device.get_image_memory_requirements(image) };
        let allocation = self.allocator.allocate(&AllocationCreateDesc {
            name: "new image memory",
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: true,
        })?;
        Ok((image, allocation))
    }
    pub fn new_uniform(
        &mut self,
        core: &mut Core,
        present_images: &PresentImage,
        data: Vec<u8>,
    ) -> Result<UniformAllocation> {
        let layout_binding = [*vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_binding);
        let layouts = unsafe {
            let t = core
                .device
                .create_descriptor_set_layout(&layout_info, None)?;
            (0..present_images.num_swapchain_images())
                .map(|_| t.clone())
                .collect::<Vec<_>>()
        };
        let pool_sizes = [*vk::DescriptorPoolSize::builder()
            .descriptor_count(present_images.num_swapchain_images() as u32)
            .ty(vk::DescriptorType::UNIFORM_BUFFER)];
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(present_images.num_swapchain_images() as u32)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
        let descriptor_pool =
            unsafe { core.device.create_descriptor_pool(&pool_create_info, None) }?;
        let mut buffer_memory = (0..present_images.num_swapchain_images())
            .map(|_| {
                self.create_buffer(
                    core,
                    data.len() as u64,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::SharingMode::EXCLUSIVE,
                    MemoryLocation::CpuToGpu,
                )
                .expect("failed to create buffer")
            })
            .collect::<Vec<_>>();
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe {
            core.device
                .allocate_descriptor_sets(&descriptor_set_alloc_info)
        }?;
        let buffer_info: Vec<[vk::DescriptorBufferInfo; 1]> = buffer_memory
            .iter()
            .map(|(buffer, _)| {
                [vk::DescriptorBufferInfo::builder()
                    .buffer(*buffer)
                    .offset(0)
                    .range(data.len() as u64)
                    .build()]
            })
            .collect();
        for i in 0..descriptor_sets.len() {
            let write = [*vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_info[i])];
            unsafe {
                core.device.update_descriptor_sets(&write, &[]);
            }
        }
        for (_buffer, allocation) in buffer_memory.iter() {
            let len = data.len();
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr() as *const std::ffi::c_void,
                    allocation
                        .mapped_ptr()
                        .expect("failed to map memory")
                        .as_ptr(),
                    len,
                );
            }
        }
        let buffers = buffer_memory
            .drain(..)
            .zip(descriptor_sets)
            .map(|((buffer, memory), descriptor_set)| (buffer, memory, descriptor_set))
            .collect();
        Ok(UniformAllocation {
            layouts,
            buffers,
            descriptor_pool,
        })
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
    pub buffer: vk::Buffer,
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
pub struct TextureAllocation {
    sampler: vk::Sampler,
    image_view: vk::ImageView,
    transfer_allocation: SubAllocation,
    buffer: vk::Buffer,
    image: vk::Image,
    image_allocation: SubAllocation,
    pub descriptor_set: vk::DescriptorSet,
}
impl TextureAllocation {
    pub fn transition_image_layout(
        core: &mut Core,
        command_pool: &mut CommandPool,
        image: &vk::Image,
        aspect_mask: vk::ImageAspectFlags,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let mut barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(*image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            );
        let (source_stage, dest_stage) = if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            (
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            )
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
            (
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            )
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        {
            barrier.src_access_mask = vk::AccessFlags::empty();
            barrier.dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
            (
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
        } else {
            panic!("unsupported layout transition")
        };
        unsafe {
            let buffer = command_pool.create_onetime_buffer(core);
            buffer.core.device.cmd_pipeline_barrier(
                buffer.command_buffer[0],
                source_stage,
                dest_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier.build()],
            );
        }
    }
}
pub struct UniformAllocation {
    pub layouts: Vec<vk::DescriptorSetLayout>,
    pub buffers: Vec<(vk::Buffer, SubAllocation, vk::DescriptorSet)>,
    descriptor_pool: vk::DescriptorPool,
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
