use super::{Device, Framebuffer, GraphicsPipeline, UniformBuffer, VertexBuffer};
use ash::{version::DeviceV1_0, vk};
use nalgebra::Matrix4;
pub struct CommandQueue {
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
    render_finished_semaphore: vk::Semaphore,
    image_available_semaphore: vk::Semaphore,
}
impl CommandQueue {
    pub fn new(
        device: &mut Device,
        graphics_pipeline: &mut GraphicsPipeline,
        framebuffers: &mut Framebuffer,
        vertex_buffers: &VertexBuffer,
        uniform: &UniformBuffer<{ std::mem::size_of::<Matrix4<f32>>() }>,
        width: u32,
        height: u32,
    ) -> Self {
        let command_pool_create_info =
            vk::CommandPoolCreateInfo::builder().queue_family_index(device.queue_family_index);
        let command_pool = unsafe {
            device
                .device
                .create_command_pool(&command_pool_create_info, None)
                .expect("failed to create command pool")
        };
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(framebuffers.framebuffers.len() as u32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);
        let command_buffers = unsafe {
            device
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("failed to allocate command buffer")
        };

        for (i, (command_buffer, framebuffer)) in command_buffers
            .iter()
            .zip(framebuffers.framebuffers.iter())
            .enumerate()
        {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();
            unsafe {
                device
                    .device
                    .begin_command_buffer(*command_buffer, &command_buffer_begin_info)
                    .expect("failed to create command buffer");
                let renderpass_info = vk::RenderPassBeginInfo::builder()
                    .render_pass(graphics_pipeline.renderpass)
                    .framebuffer(*framebuffer)
                    .render_area(vk::Rect2D {
                        extent: vk::Extent2D { width, height },
                        offset: vk::Offset2D { x: 0, y: 0 },
                    })
                    .clear_values(&[vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.1, 0.1, 0.1, 1.0],
                        },
                    }]);
                device.device.cmd_begin_render_pass(
                    *command_buffer,
                    &renderpass_info,
                    vk::SubpassContents::INLINE,
                );
                device.device.cmd_bind_pipeline(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline.graphics_pipeline,
                );
                device.device.cmd_bind_vertex_buffers(
                    *command_buffer,
                    0,
                    &[vertex_buffers.buffer],
                    &[0],
                );
                device.device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline.pipeline_layout,
                    0,
                    &[uniform.buffers[i].2],
                    &[],
                );
                device.device.cmd_draw(*command_buffer, 3, 1, 0, 0);
                device.device.cmd_end_render_pass(*command_buffer);
                device
                    .device
                    .end_command_buffer(*command_buffer)
                    .expect("failed to create command buffer");
            };
        }
        let fences: Vec<vk::Fence> = command_buffers
            .iter()
            .map(|_| {
                let fence_create_info =
                    vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
                unsafe {
                    device
                        .device
                        .create_fence(&fence_create_info, None)
                        .expect("failed to create fence")
                }
            })
            .collect();
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        let image_available_semaphore =
            unsafe { device.device.create_semaphore(&semaphore_create_info, None) }
                .expect("failed to create semaphore");
        let render_finished_semaphore =
            unsafe { device.device.create_semaphore(&semaphore_create_info, None) }
                .expect("failed to create semaphore");
        Self {
            command_pool,
            command_buffers,
            fences,
            image_available_semaphore,
            render_finished_semaphore,
        }
    }
    pub unsafe fn render_frame(&mut self, device: &mut Device) {
        let (image_index, _) = device
            .swapchain_loader
            .acquire_next_image(
                device.swapchain,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )
            .expect("failed to aquire image");
        device
            .device
            .wait_for_fences(&[self.fences[image_index as usize]], true, u64::MAX)
            .expect("failed to wait for fence");
        device
            .device
            .reset_fences(&[self.fences[image_index as usize]])
            .expect("failed to reset fence");

        let signal_semaphores = [self.render_finished_semaphore];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.image_available_semaphore])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[self.command_buffers[image_index as usize]])
            .signal_semaphores(&signal_semaphores)
            .build();
        device
            .device
            .queue_submit(
                device.present_queue,
                &[submit_info],
                self.fences[image_index as usize],
            )
            .expect("failed to submit queue");
        let wait_semaphores = [device.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&wait_semaphores)
            .image_indices(&image_indices);
        device
            .swapchain_loader
            .queue_present(device.present_queue, &present_info)
            .expect("failed to present queue");
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            device
                .device
                .wait_for_fences(&self.fences, true, 10000000)
                .expect("failed to wait for fence");
            device
                .device
                .device_wait_idle()
                .expect("failed to wait idle");
            device
                .device
                .destroy_semaphore(self.render_finished_semaphore, None);
            device
                .device
                .destroy_semaphore(self.image_available_semaphore, None);
            for fence in self.fences.iter() {
                device.device.destroy_fence(*fence, None);
            }
            // as Vulkan spec all command pools are freed
            device.device.destroy_command_pool(self.command_pool, None);
        }
    }
}
