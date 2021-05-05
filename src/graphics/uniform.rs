use super::{find_memorytype_index, Device, PresentImage};
use ash::{version::DeviceV1_0, vk};
/// For now only uniform buffer should be allocated at a time
pub struct UniformBuffer<const SIZE: usize> {
    pub layout: Vec<vk::DescriptorSetLayout>,
    pub buffers: Vec<(vk::Buffer, vk::DeviceMemory, vk::DescriptorSet)>,

    descriptor_pool: vk::DescriptorPool,
}
impl<const SIZE: usize> UniformBuffer<SIZE> {
    pub fn new(device: &mut Device, present_image: &PresentImage) -> Self {
        let layout_binding = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .build()];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_binding);

        let layout: Vec<vk::DescriptorSetLayout> = unsafe {
            let temp_layout = device
                .device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create layout");
            (0..present_image.num_swapchain_images())
                .map(|_| temp_layout.clone())
                .collect()
        };
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .descriptor_count(present_image.num_swapchain_images() as u32)
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .build()];
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(present_image.num_swapchain_images() as u32)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

        let descriptor_pool = unsafe {
            device
                .device
                .create_descriptor_pool(&pool_create_info, None)
                .expect("failed to create pool")
        };
        let buffers_memory: Vec<(vk::Buffer, vk::DeviceMemory)> = (0..present_image
            .num_swapchain_images())
            .map(|_| device.create_buffer(SIZE as u64, vk::BufferUsageFlags::UNIFORM_BUFFER))
            .collect();
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layout);
        let descriptor_sets = unsafe {
            device
                .device
                .allocate_descriptor_sets(&descriptor_set_alloc_info)
        }
        .expect("failed to allocate layout");
        let buffer_info: Vec<[vk::DescriptorBufferInfo; 1]> = buffers_memory
            .iter()
            .map(|(buffer, _)| {
                [vk::DescriptorBufferInfo::builder()
                    .buffer(*buffer)
                    .offset(0)
                    .range(SIZE as u64)
                    .build()]
            })
            .collect();
        for i in 0..descriptor_sets.len() {
            let write = [vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_info[i])
                .build()];
            unsafe {
                device.device.update_descriptor_sets(&write, &[]);
            }
        }

        let buffers = buffers_memory
            .iter()
            .zip(descriptor_sets)
            .map(|((buffer, memory), descriptor_set)| (*buffer, *memory, descriptor_set))
            .collect();
        Self {
            layout,
            buffers,
            descriptor_pool,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            let mut sets = vec![];
            sets.reserve(self.buffers.len());
            for (buffer, memory, descriptor_set) in self.buffers.iter() {
                device.device.free_memory(*memory, None);
                device.device.destroy_buffer(*buffer, None);
                sets.push(*descriptor_set);
            }
            device
                .device
                .free_descriptor_sets(self.descriptor_pool, &sets)
                .expect("failed to free descriptor set");
            device
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            for layout in self.layout.iter() {
                device.device.destroy_descriptor_set_layout(*layout, None);
            }
        }
    }
}
