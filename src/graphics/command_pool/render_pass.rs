use super::{
    CommandPool, Device, Framebuffer, GraphicsPipeline, IndexBuffer, Texture, UniformBuffer,
    VertexBuffer,
};
use ash::{version::DeviceV1_0, vk};
use nalgebra::Matrix4;
use std::collections::HashMap;
mod semaphore_buffer;
use semaphore_buffer::SemaphoreBuffer;
enum ClearOp {
    ClearColor,
    DoNotClear,
}
pub struct RenderMesh<'a> {
    pub uniform_data: HashMap<String, *const std::ffi::c_void>,
    pub view_matrix: Matrix4<f32>,
    pub vertex_buffer: &'a VertexBuffer,
    pub index_buffer: &'a IndexBuffer,
    pub texture: &'a Texture,
    pub offsets: OffsetData,
}

pub struct TempRenderMesh<'a> {
    pub view_matrix: Matrix4<f32>,
    pub vertex_buffer: &'a VertexBuffer,
    pub index_buffer: &'a IndexBuffer,
    pub texture: &'a Texture,
    pub offsets: OffsetData,
}
//collection of data used for rendering
pub struct RenderCollection<'a> {
    //orders data by submission of uniform
    batches: HashMap<(String, &'a [u8]), TempRenderMesh<'a>>,
}
impl<'a> RenderCollection<'a> {}
#[derive(Clone, Copy)]
// Offset of mesh to draw
pub struct OffsetData {
    /// offset in `std::mem::size_of::<f32>()*3*indicies`
    pub vertex_offset: usize,
    /// offset in `std::mem::size_of::<u32>()*1*indicies`
    pub index_offset: usize,
}
pub struct RenderPass {
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
    semaphore_buffer: SemaphoreBuffer,
    render_finished_semaphore: vk::Semaphore,
    image_available_semaphore: vk::Semaphore,
}
impl RenderPass {
    pub fn new(
        device: &mut Device,
        command_queue: &CommandPool,
        framebuffers: &Framebuffer,
    ) -> Self {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(framebuffers.framebuffers.len() as u32)
            .command_pool(command_queue.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);
        let command_buffers = unsafe {
            device
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("failed to allocate command buffer")
        };

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
        let semaphore_buffer =
            SemaphoreBuffer::new(image_available_semaphore, render_finished_semaphore);
        Self {
            command_buffers,
            fences,
            semaphore_buffer,
            image_available_semaphore,
            render_finished_semaphore,
        }
    }
    // Builds renderpass using selected uniforms. Waits for selected Fence
    unsafe fn build_renderpass(
        &mut self,
        device: &mut Device,
        framebuffer: &vk::Framebuffer,
        graphics_pipeline: &GraphicsPipeline,
        width: u32,
        height: u32,
        image_index: usize,
        uniform_buffers: &HashMap<String, UniformBuffer>,
        mesh: &RenderMesh,
        clear_op: ClearOp,
    ) {
        let begin_info = vk::CommandBufferBeginInfo::builder();
        device
            .device
            .begin_command_buffer(self.command_buffers[image_index], &begin_info)
            .expect("failed to build command buffer");

        let renderpass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(match clear_op {
                ClearOp::ClearColor => graphics_pipeline.clear_pipeline.renderpass,
                ClearOp::DoNotClear => graphics_pipeline.load_pipeline.renderpass,
            })
            .framebuffer(*framebuffer)
            .render_area(vk::Rect2D {
                extent: vk::Extent2D { width, height },
                offset: vk::Offset2D { x: 0, y: 0 },
            })
            .clear_values(&[
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.1, 0.1, 0.1, 0.1],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ]);
        device.device.cmd_begin_render_pass(
            self.command_buffers[image_index],
            &renderpass_info,
            vk::SubpassContents::INLINE,
        );
        device.device.cmd_bind_pipeline(
            self.command_buffers[image_index],
            vk::PipelineBindPoint::GRAPHICS,
            match clear_op {
                ClearOp::ClearColor => graphics_pipeline.clear_pipeline.graphics_pipeline,
                ClearOp::DoNotClear => graphics_pipeline.load_pipeline.graphics_pipeline,
            },
        );
        device.device.cmd_bind_vertex_buffers(
            self.command_buffers[image_index],
            0,
            &[mesh.vertex_buffer.buffer],
            &[0],
        );
        device.device.cmd_bind_index_buffer(
            self.command_buffers[image_index],
            mesh.index_buffer.buffer,
            0,
            vk::IndexType::UINT32,
        );
        let mut descriptor_sets = uniform_buffers
            .iter()
            .map(|(_key, uniform)| uniform.buffers[image_index].2)
            .collect::<Vec<_>>();
        descriptor_sets.push(mesh.texture.descriptor_set);
        device.device.cmd_bind_descriptor_sets(
            self.command_buffers[image_index],
            vk::PipelineBindPoint::GRAPHICS,
            graphics_pipeline.pipeline_layout,
            0,
            &descriptor_sets,
            &[],
        );
        //getting the slice
        let matrix_ptr = mesh.view_matrix.as_ptr() as *const u8;
        let matrix_slice =
            std::slice::from_raw_parts(matrix_ptr, std::mem::size_of::<Matrix4<f32>>());
        device.device.cmd_push_constants(
            self.command_buffers[image_index],
            graphics_pipeline.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            matrix_slice,
        );
        device.device.cmd_draw_indexed(
            self.command_buffers[image_index],
            mesh.index_buffer.get_num_indicies() as u32,
            1,
            mesh.offsets.index_offset as u32,
            mesh.offsets.vertex_offset as i32,
            0,
        );
        device
            .device
            .cmd_end_render_pass(self.command_buffers[image_index]);
        device
            .device
            .end_command_buffer(self.command_buffers[image_index])
            .expect("failed to create command buffer");
        device
            .device
            .reset_fences(&[self.fences[image_index as usize]])
            .expect("failed to reset fence");
    }
    pub unsafe fn render_frame(
        &mut self,
        device: &mut Device,
        framebuffer: &Framebuffer,
        graphics_pipeline: &GraphicsPipeline,
        width: u32,
        height: u32,
        uniform_buffers: &mut HashMap<String, UniformBuffer>,
        meshes: &mut [RenderMesh],
    ) {
        let (image_index, _) = device
            .swapchain_loader
            .acquire_next_image(
                device.swapchain,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )
            .expect("failed to aquire image");
        let semaphores = self.semaphore_buffer.get_semaphores(device, meshes.len());
        for (idx, (mesh, semaphore)) in meshes.iter_mut().zip(semaphores).enumerate() {
            device
                .device
                .wait_for_fences(&[self.fences[image_index as usize]], true, u64::MAX)
                .expect("failed to wait for fence");
            device
                .device
                .reset_fences(&[self.fences[image_index as usize]])
                .expect("failed to reset fence");
            for (key, data) in mesh.uniform_data.iter() {
                uniform_buffers
                    .get_mut(key)
                    .expect("failed to find uniform \"view\"")
                    .update_uniform(device, image_index as usize, *data);
            }
            self.build_renderpass(
                device,
                &framebuffer.framebuffers[image_index as usize],
                graphics_pipeline,
                width,
                height,
                image_index as usize,
                uniform_buffers,
                mesh,
                match idx {
                    0 => ClearOp::ClearColor,
                    _ => ClearOp::DoNotClear,
                },
            );
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[semaphore.start_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.command_buffers[image_index as usize]])
                .signal_semaphores(&[semaphore.finished_semaphore])
                .build();

            device
                .device
                .queue_submit(
                    device.present_queue,
                    &[submit_info],
                    self.fences[image_index as usize],
                )
                .expect("failed to submit queue");
        }
        let wait_semaphores = [device.swapchain];
        let image_indices = [image_index];
        let present_wait_semaphore = [self.semaphore_buffer.render_finished_semaphore()];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&present_wait_semaphore)
            .swapchains(&wait_semaphores)
            .image_indices(&image_indices);
        device
            .swapchain_loader
            .queue_present(device.present_queue, &present_info)
            .expect("failed to present queue");
    }
    pub fn wait_idle(&mut self, device: &mut Device) {
        unsafe {
            device
                .device
                .wait_for_fences(&self.fences, true, 10000000)
                .expect("failed to wait for fence");
            device
                .device
                .device_wait_idle()
                .expect("failed to wait idle");
        }
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
            self.semaphore_buffer.free(device);
            for fence in self.fences.iter() {
                device.device.destroy_fence(*fence, None);
            }
        }
    }
}
