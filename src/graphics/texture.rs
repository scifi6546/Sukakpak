use super::{find_memorytype_index, CommandPool, DescriptorSets, Device, PresentImage};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use image::io::Reader as ImageReader;
pub struct Texture {
    image_data: image::RgbaImage,
    sampler: vk::Sampler,
    image_view: vk::ImageView,
    transfer_memory: vk::DeviceMemory,
    buffer: vk::Buffer,
    image: vk::Image,
    image_memory: vk::DeviceMemory,
    descriptor_set: vk::DescriptorSet,
}
impl Texture {
    fn new(
        device: &mut Device,
        command_queue: &mut CommandPool,
        texture_pool: &TexturePool,
        texture_creator: &TextureCreator,
    ) -> Self {
        let image_data = ImageReader::open("screenshot.png")
            .expect("failed to load image")
            .decode()
            .expect("failed to decode image")
            .to_rgba8();

        let (buffer, transfer_memory) = device.create_buffer(
            image_data.as_raw().len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::SharingMode::EXCLUSIVE,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        unsafe {
            let memory_ptr = device
                .device
                .map_memory(
                    transfer_memory,
                    0,
                    image_data.as_raw().len() as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory");
            std::ptr::copy_nonoverlapping(
                image_data.as_raw().as_ptr() as *const std::ffi::c_void,
                memory_ptr,
                image_data.as_raw().len(),
            );
            device.device.unmap_memory(transfer_memory);
        }

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(
                vk::Extent3D::builder()
                    .width(image_data.width())
                    .height(image_data.height())
                    .depth(1)
                    .build(),
            )
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);
        let image = unsafe { device.device.create_image(&image_create_info, None) }
            .expect("failed to create image");
        let memory_reqirements = unsafe { device.device.get_image_memory_requirements(image) };
        let memory_alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_reqirements.size)
            .memory_type_index(
                find_memorytype_index(
                    &memory_reqirements,
                    &device.memory_properties,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )
                .expect("failed to find memory"),
            );
        let image_memory = unsafe { device.device.allocate_memory(&memory_alloc_info, None) }
            .expect("failed to allocate device memory");
        unsafe {
            device
                .device
                .bind_image_memory(image, image_memory, 0)
                .expect("failed to bind memory");
        }

        Self::transition_image_layout(
            device,
            command_queue,
            &image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        Self::copy_buffer_image(
            device,
            command_queue,
            image,
            buffer,
            image_data.width(),
            image_data.height(),
        );
        Self::transition_image_layout(
            device,
            command_queue,
            &image,
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
        let image_view = unsafe { device.device.create_image_view(&view_info, None) }
            .expect("failed to create view");
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
                    device
                        .instance
                        .get_physical_device_properties(device.physical_device)
                }
                .limits
                .max_sampler_anisotropy,
            );
        let sampler = unsafe { device.device.create_sampler(&sampler_info, None) }
            .expect("failed to create sampler");
        let layouts = [texture_creator.layout];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(texture_pool.descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_set = unsafe {
            device
                .device
                .allocate_descriptor_sets(&descriptor_set_alloc_info)
        }
        .expect("failed to allocate descriptor set")[0];
        let image_descriptor_info = [*vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(sampler)];
        let write = [*vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_descriptor_info)];
        unsafe {
            device.device.update_descriptor_sets(&write, &[]);
        }
        Self {
            image,
            sampler,
            image_data,
            image_view,
            buffer,
            transfer_memory,
            image_memory,
            descriptor_set,
        }
    }
    pub fn bind_image(&mut self) {
        let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    }
    fn transition_image_layout(
        device: &mut Device,
        command_queue: &mut CommandPool,
        image: &vk::Image,
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
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
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
        } else {
            panic!("unsupported layout transition")
        };
        unsafe {
            let buffer = command_queue.create_onetime_buffer(device);
            buffer.device.device.cmd_pipeline_barrier(
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
    fn copy_buffer_image(
        device: &mut Device,
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
            let command_buffer = command_queue.create_onetime_buffer(device);
            command_buffer.device.device.cmd_copy_buffer_to_image(
                command_buffer.command_buffer[0],
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }
    }
    pub fn free(&mut self, device: &mut Device, texture_pool: &TexturePool) {
        unsafe {
            device
                .device
                .free_descriptor_sets(texture_pool.descriptor_pool, &[self.descriptor_set])
                .expect("failed to free");
            device.device.destroy_sampler(self.sampler, None);
            device.device.destroy_image_view(self.image_view, None);
            device.device.free_memory(self.image_memory, None);
            device.device.destroy_image(self.image, None);
            device.device.free_memory(self.transfer_memory, None);
            device.device.destroy_buffer(self.buffer, None);
        }
    }
}

pub struct TextureCreator {
    layout: vk::DescriptorSetLayout,
}
impl TextureCreator {
    pub fn new(device: &mut Device) -> Self {
        let layout_binding = [*vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let layout_create_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_binding);
        let layout = unsafe {
            device
                .device
                .create_descriptor_set_layout(&layout_create_info, None)
        }
        .expect("failed to create layout");
        Self { layout }
    }
    pub fn free(&self, device: &mut Device) {
        unsafe {
            device
                .device
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
impl DescriptorSets for TextureCreator {
    fn get_layouts(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.layout]
    }
}
pub struct TexturePool {
    descriptor_pool: vk::DescriptorPool,
}
impl TexturePool {
    pub fn new(
        device: &mut Device,
        command_pool: &mut CommandPool,
        textures: &Vec<TextureCreator>,
        swapchain: &PresentImage,
    ) -> (Self, Vec<Texture>) {
        let pool_sizes = [*vk::DescriptorPoolSize::builder()
            .descriptor_count((swapchain.num_swapchain_images() * textures.len()) as u32)
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)];
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets((swapchain.num_swapchain_images() * textures.len()) as u32)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
        let descriptor_pool = unsafe {
            device
                .device
                .create_descriptor_pool(&pool_create_info, None)
                .expect("failed to create pool")
        };
        let texture_pool = Self { descriptor_pool };
        let texture_array = textures
            .iter()
            .map(|creator| Texture::new(device, command_pool, &texture_pool, creator))
            .collect();
        (texture_pool, texture_array)
    }
    pub fn free(&self, device: &mut Device) {
        unsafe {
            device
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
