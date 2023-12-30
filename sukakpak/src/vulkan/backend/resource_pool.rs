use super::{CommandPool, Core, ShaderDescription, VertexLayout};
use anyhow::Result;
use ash::{vk, Device, Instance};
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc},
    AllocationSizes, MemoryLocation,
};
use image::RgbaImage;
use nalgebra::Vector2;
mod descriptor_pool;
use descriptor_pool::DescriptorPool;
pub use descriptor_pool::{DescriptorDesc, DescriptorName};
use std::mem::{size_of, ManuallyDrop};
pub struct ResourcePool {
    allocator: ManuallyDrop<Allocator>,
    texture_descriptor_pool: DescriptorPool,
    sampler_descriptor_pool: DescriptorPool,
}
impl ResourcePool {
    pub fn new(core: &Core, shader: &ShaderDescription) -> Result<Self> {
        let texture_layouts = shader
            .textures
            .iter()
            .map(|(name, texture)| {
                (
                    name.clone(),
                    DescriptorDesc {
                        layout_binding: texture.image_layout_binding.clone(),
                    },
                )
            })
            .collect();
        let sampler_layouts = shader
            .textures
            .iter()
            .map(|(name, texture)| {
                (
                    name.clone(),
                    DescriptorDesc {
                        layout_binding: texture.sampler_layout_binding.clone(),
                    },
                )
            })
            .collect();
        Ok(Self {
            allocator: ManuallyDrop::new(
                Allocator::new(&AllocatorCreateDesc {
                    instance: core.instance.clone(),
                    device: core.device.clone(),
                    physical_device: core.physical_device,
                    buffer_device_address: false,
                    debug_settings: Default::default(),
                    allocation_sizes: AllocationSizes::default(),
                })
                .expect("failed to create allocator"),
            ),
            texture_descriptor_pool: DescriptorPool::new(
                core,
                vk::DescriptorType::SAMPLED_IMAGE,
                &texture_layouts,
            )?,
            sampler_descriptor_pool: DescriptorPool::new(
                core,
                vk::DescriptorType::SAMPLER,
                &sampler_layouts,
            )?,
        })
    }
    pub fn allocate_vertex_buffer(
        &mut self,
        core: &mut Core,
        mesh: Vec<u8>,
        layout: VertexLayout,
    ) -> Result<VertexBufferAllocation> {
        let (binding_description, input_description) = (
            *vk::VertexInputBindingDescription::builder()
                .binding(0)
                .input_rate(vk::VertexInputRate::VERTEX)
                .stride(layout.components.iter().map(|c| c.size()).sum::<usize>() as u32),
            layout
                .components
                .iter()
                .enumerate()
                .map(|(i, comp)| {
                    *vk::VertexInputAttributeDescription::builder()
                        .binding(0)
                        .location(i as u32)
                        .format(comp.into())
                })
                .collect(),
        );
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
            allocation_scheme: AllocationScheme::DedicatedBuffer(buffer),
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
            allocation: Some(allocation),
            buffer,
            binding_description,
            input_description,
        })
    }
    pub fn free_allocation(&mut self, allocation: Allocation) -> Result<()> {
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
            allocation: Some(allocation),
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
    ) -> Result<(vk::Buffer, Allocation)> {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(sharing_mode);
        let buffer = unsafe { core.device.create_buffer(&buffer_create_info, None)? };
        let requirements = unsafe { core.device.get_buffer_memory_requirements(buffer) };
        let allocation = self.allocator.allocate(&AllocationCreateDesc {
            name: "general buffer",
            requirements,
            location: memory_location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            core.device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
        }
        Ok((buffer, allocation))
    }
    pub fn new_image(
        &mut self,
        core: &mut Core,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        dimensions: Vector2<u32>,
    ) -> Result<(vk::Image, Allocation)> {
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
            allocation_scheme: AllocationScheme::DedicatedImage(image),
        })?;
        unsafe {
            core.device
                .bind_image_memory(image, allocation.memory(), allocation.offset())?
        };
        Ok((image, allocation))
    }
    pub fn allocate_texture(
        &mut self,
        core: &mut Core,
        command_pool: &mut CommandPool,
        image_data: &RgbaImage,
    ) -> Result<TextureAllocation> {
        let image_len = image_data.as_raw().len();
        let (buffer, transfer_allocation) = self.create_buffer(
            core,
            image_len as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::SharingMode::EXCLUSIVE,
            MemoryLocation::CpuToGpu,
        )?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                image_data.as_raw().as_ptr() as *const std::ffi::c_void,
                transfer_allocation
                    .mapped_ptr()
                    .expect("failed to map texture pointer")
                    .as_ptr(),
                image_len,
            );
        }
        let (image, image_allocation) = self.new_image(
            core,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            Vector2::new(image_data.width(), image_data.height()),
        )?;
        TextureAllocation::transition_image_layout(
            core,
            command_pool,
            &image,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        TextureAllocation::copy_buffer_image(
            core,
            command_pool,
            image,
            buffer,
            image_data.width(),
            image_data.height(),
        );
        TextureAllocation::transition_image_layout(
            core,
            command_pool,
            &image,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(
                *vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );
        let image_view = unsafe { core.device.create_image_view(&view_info, None) }?;
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0)
            .max_anisotropy(
                unsafe {
                    core.instance
                        .get_physical_device_properties(core.physical_device)
                }
                .limits
                .max_sampler_anisotropy,
            );
        let sampler = unsafe { core.device.create_sampler(&sampler_info, None) }
            .expect("failed to create sampler");
        let descriptor_sets = self.get_texture_descriptor(
            core,
            image_view,
            sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;
        Ok(TextureAllocation {
            buffer,
            descriptor_sets,
            image,
            image_allocation,
            image_view,
            sampler,
            transfer_allocation,
        })
    }
    pub fn get_texture_descriptor(
        &mut self,
        core: &mut Core,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        layout: vk::ImageLayout,
    ) -> Result<TextureDescriptorSets> {
        let texture_descriptor_set = unsafe {
            self.texture_descriptor_pool
                .allocate_descriptor_set(core, "mesh_texture")
        }?[0];
        let descriptor_image_info = [
            *vk::DescriptorImageInfo::builder()
                .image_layout(layout)
                .image_view(image_view), //.sampler(sampler)
        ];
        let descriptor_write = [*vk::WriteDescriptorSet::builder()
            .dst_set(texture_descriptor_set)
            .dst_binding(
                self.texture_descriptor_pool
                    .get_descriptor_desc("mesh_texture")
                    .unwrap()
                    .layout_binding
                    .binding,
            )
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
            .image_info(&descriptor_image_info)];
        unsafe {
            core.device.update_descriptor_sets(&descriptor_write, &[]);
        }

        let sampler_descriptor_set = unsafe {
            self.sampler_descriptor_pool
                .allocate_descriptor_set(core, "mesh_texture")
        }?[0];
        let descriptor_image_info = [*vk::DescriptorImageInfo::builder().sampler(sampler)];
        let descriptor_write = [*vk::WriteDescriptorSet::builder()
            .dst_set(sampler_descriptor_set)
            .dst_binding(
                self.sampler_descriptor_pool
                    .get_descriptor_desc("mesh_texture")
                    .unwrap()
                    .layout_binding
                    .binding,
            )
            .image_info(&descriptor_image_info)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::SAMPLER)];
        unsafe {
            core.device.update_descriptor_sets(&descriptor_write, &[]);
        }
        Ok(TextureDescriptorSets {
            texture_descriptor_set,
            sampler_descriptor_set,
        })
    }
    /// Allocates descriptor sets. pool provided by descriptor pool
    pub fn get_descriptor_set_layouts(&self) -> Vec<vk::DescriptorSetLayout> {
        self.texture_descriptor_pool
            .get_descriptor_layouts()
            .iter()
            .zip(self.sampler_descriptor_pool.get_descriptor_layouts().iter())
            .map(|(l1, l2)| [l1, l2])
            .flatten()
            .copied()
            .collect()
    }
    pub fn free(&mut self, core: &mut Core) -> Result<()> {
        self.texture_descriptor_pool.free(core)?;
        self.sampler_descriptor_pool.free(core)?;
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct TextureDescriptorSets {
    pub texture_descriptor_set: vk::DescriptorSet,
    pub sampler_descriptor_set: vk::DescriptorSet,
}
pub struct IndexBufferAllocation {
    pub buffer: vk::Buffer,
    pub allocation: Option<Allocation>,
    pub buffer_size: usize,
}
impl IndexBufferAllocation {
    pub fn num_indices(&self) -> usize {
        self.buffer_size / size_of::<u32>()
    }
    pub fn free(mut self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        resource_pool
            .allocator
            .free(self.allocation.take().expect("index buffer already freed"))?;
        unsafe {
            core.device.destroy_buffer(self.buffer, None);
        }
        Ok(())
    }
}
pub struct VertexBufferAllocation {
    allocation: Option<Allocation>,
    pub buffer: vk::Buffer,
    pub binding_description: vk::VertexInputBindingDescription,
    pub input_description: Vec<vk::VertexInputAttributeDescription>,
}
impl VertexBufferAllocation {
    pub fn free(mut self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        resource_pool
            .allocator
            .free(self.allocation.take().expect("vertex buffer already freed"))?;
        unsafe {
            core.device.destroy_buffer(self.buffer, None);
        }
        Ok(())
    }
}
pub struct TextureAllocation {
    sampler: vk::Sampler,
    image_view: vk::ImageView,
    transfer_allocation: Allocation,
    buffer: vk::Buffer,
    image: vk::Image,
    image_allocation: Allocation,
    pub descriptor_sets: TextureDescriptorSets,
}
impl TextureAllocation {
    fn copy_buffer_image(
        core: &mut Core,
        command_queue: &mut CommandPool,
        image: vk::Image,
        buffer: vk::Buffer,
        width: u32,
        height: u32,
    ) {
        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                *vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                height,
                width,
                depth: 1,
            })
            .build();
        unsafe {
            let command_buffer = command_queue.create_onetime_buffer(core);
            command_buffer.core.device.cmd_copy_buffer_to_image(
                command_buffer.command_buffer[0],
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }
    }
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
        } else if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::GENERAL
        {
            barrier.src_access_mask = vk::AccessFlags::empty();
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
            (
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
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
    pub fn free(self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        unsafe {
            core.device.destroy_sampler(self.sampler, None);
            core.device.destroy_image_view(self.image_view, None);
            resource_pool.free_allocation(self.image_allocation)?;

            core.device.free_descriptor_sets(
                resource_pool.texture_descriptor_pool.descriptor_pool,
                &[self.descriptor_sets.texture_descriptor_set],
            )?;
            core.device.free_descriptor_sets(
                resource_pool.sampler_descriptor_pool.descriptor_pool,
                &[self.descriptor_sets.sampler_descriptor_set],
            )?;
            core.device.destroy_image(self.image, None);
            resource_pool.free_allocation(self.transfer_allocation)?;
            core.device.destroy_buffer(self.buffer, None);
        }
        Ok(())
    }
}
