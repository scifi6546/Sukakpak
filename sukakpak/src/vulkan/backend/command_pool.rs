use super::Core;
use ash::{version::DeviceV1_0, vk};
pub struct OneTimeCommandBuffer<'a> {
    pub core: &'a Core,
    pub command_buffer: [vk::CommandBuffer; 1],
    command_queue: &'a CommandPool,
}

impl<'a> Drop for OneTimeCommandBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            self.core
                .device
                .end_command_buffer(self.command_buffer[0])
                .expect("failed to end command buer");
            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&self.command_buffer)
                .build();
            self.core
                .device
                .queue_submit(self.core.present_queue, &[submit_info], vk::Fence::null())
                .expect("failed to submit queue");
            self.core
                .device
                .queue_wait_idle(self.core.present_queue)
                .expect("failed to wait idle");
            self.core
                .device
                .free_command_buffers(self.command_queue.command_pool, &self.command_buffer);
        }
    }
}
pub struct CommandPool {
    pub command_pool: vk::CommandPool,
}
impl CommandPool {
    pub fn new(core: &mut Core) -> Self {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(core.queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe {
            core.device
                .create_command_pool(&command_pool_create_info, None)
                .expect("failed to create command pool")
        };
        Self { command_pool }
    }
    pub unsafe fn create_onetime_buffer<'a>(
        &'a mut self,
        core: &'a mut Core,
    ) -> OneTimeCommandBuffer<'a> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pool)
            .command_buffer_count(1);
        let command_buffer = [core
            .device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("failed to allocate command buffer")[0]];
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        core.device
            .begin_command_buffer(command_buffer[0], &begin_info)
            .expect("failed to begin command buffer");
        OneTimeCommandBuffer {
            command_buffer,
            core,
            command_queue: self,
        }
    }
    pub fn free(&mut self, core: &mut Core) {
        unsafe {
            // as Vulkan spec all command pools are freed
            core.device.destroy_command_pool(self.command_pool, None);
        }
    }
}
