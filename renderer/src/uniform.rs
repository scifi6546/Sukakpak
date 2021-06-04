use super::{DescriptorSets, Device, FreeChecker, PresentImage, UniformDescription};
use ash::{version::DeviceV1_0, vk};
/// For now only uniform buffer should be allocated at a time
pub struct UniformBuffer {
    pub layout: Vec<vk::DescriptorSetLayout>,
    pub buffers: Vec<(vk::Buffer, vk::DeviceMemory, vk::DescriptorSet)>,
    size: usize,
    free_checker: FreeChecker,
    descriptor_pool: vk::DescriptorPool,
}
impl UniformBuffer {
    pub fn new(
        device: &mut Device,
        present_image: &PresentImage,
        uniform_description: &UniformDescription,
        data: *const std::ffi::c_void,
    ) -> Self {
        let (descriptor_pool, layout, descriptor_sets) =
            uniform_description.get_layouts(device, present_image.num_swapchain_images() as u32);
        let layout = (0..present_image.num_swapchain_images())
            .map(|_| layout.clone())
            .collect::<Vec<_>>();
        let buffers_memory: Vec<(vk::Buffer, vk::DeviceMemory)> = (0..present_image
            .num_swapchain_images())
            .map(|_| {
                device.create_buffer(
                    uniform_description.size as u64,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::SharingMode::EXCLUSIVE,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
            })
            .collect();
        let buffer_info: Vec<[vk::DescriptorBufferInfo; 1]> = buffers_memory
            .iter()
            .map(|(buffer, _)| {
                [vk::DescriptorBufferInfo::builder()
                    .buffer(*buffer)
                    .offset(0)
                    .range(uniform_description.size as u64)
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
        for (_buffer, memory) in buffers_memory.iter() {
            unsafe {
                let ptr = device
                    .device
                    .map_memory(
                        *memory,
                        0,
                        uniform_description.size as u64,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("failed to map memory");
                std::ptr::copy_nonoverlapping(data, ptr, uniform_description.size);
                device.device.unmap_memory(*memory);
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
            size: uniform_description.size,
            free_checker: FreeChecker::default(),
            descriptor_pool,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        self.free_checker.free();
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
            device
                .device
                .destroy_descriptor_set_layout(self.layout[0], None)
        }
    }
    pub unsafe fn update_uniform(&mut self, device: &mut Device, image_index: usize, data: &[u8]) {
        let ptr = device
            .device
            .map_memory(
                self.buffers[image_index].1,
                0,
                self.size as u64,
                vk::MemoryMapFlags::empty(),
            )
            .expect("failed to map memory");
        std::ptr::copy_nonoverlapping(data.as_ptr() as *const std::ffi::c_void, ptr, self.size);
        device.device.unmap_memory(self.buffers[image_index].1);
    }
}
impl DescriptorSets for UniformBuffer {
    fn get_layout(&self) -> vk::DescriptorSetLayout {
        self.layout[0]
    }
}
