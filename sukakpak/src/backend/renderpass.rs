use super::{
    CommandPool, Core, Framebuffer, GraphicsPipeline, IndexBufferAllocation, Texture,
    UniformBuffer, VertexBuffer,
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
    pub uniform_data: HashMap<String, &'a [u8]>,
    pub view_matrix: Matrix4<f32>,
    pub vertex_buffer: &'a VertexBuffer,
    pub index_buffer: &'a IndexBufferAllocation,
    pub texture: &'a Texture,
    pub offsets: OffsetData,
}

pub struct RenderCollectionMesh<'a> {
    pub view_matrix: Matrix4<f32>,
    pub vertex_buffer: &'a VertexBuffer,
    pub index_buffer: &'a IndexBufferAllocation,
    pub texture: &'a Texture,
    pub offsets: OffsetData,
}
#[derive(Default)]
//collection of data used for rendering
pub struct RenderCollection<'a> {
    //orders data by submission of uniform
    batches: HashMap<String, HashMap<Vec<u8>, Vec<RenderCollectionMesh<'a>>>>,
}

impl<'a> RenderCollection<'a> {
    pub fn num_uniforms_to_update(&self) -> usize {
        self.batches
            .iter()
            .map(|(_k, map)| map.len())
            .fold(0, |r, s| r + s)
    }
    pub fn push(&mut self, mesh: RenderMesh<'a>) {
        for (name, data) in mesh.uniform_data.iter() {
            if self.batches.contains_key(name) {
                let data_entry = self.batches.get_mut(name).unwrap();
                let data_vec = data.to_vec();
                if data_entry.contains_key(&data_vec) {
                    let v = data_entry.get_mut(&data_vec).unwrap();
                    v.push(RenderCollectionMesh {
                        view_matrix: mesh.view_matrix,
                        vertex_buffer: mesh.vertex_buffer,
                        index_buffer: mesh.index_buffer,
                        texture: mesh.texture,
                        offsets: mesh.offsets,
                    });
                } else {
                    data_entry.insert(
                        data_vec,
                        vec![RenderCollectionMesh {
                            view_matrix: mesh.view_matrix,
                            vertex_buffer: mesh.vertex_buffer,
                            index_buffer: mesh.index_buffer,
                            texture: mesh.texture,
                            offsets: mesh.offsets,
                        }],
                    );
                }
            } else {
                let mut map = HashMap::new();
                map.insert(
                    data.to_vec(),
                    vec![RenderCollectionMesh {
                        view_matrix: mesh.view_matrix,
                        vertex_buffer: mesh.vertex_buffer,
                        index_buffer: mesh.index_buffer,
                        texture: mesh.texture,
                        offsets: mesh.offsets,
                    }],
                );
                self.batches.insert(name.clone(), map);
            }
        }
    }
}
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
    pub fn new(core: &mut Core, command_pool: &CommandPool, framebuffers: &Framebuffer) -> Self {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(framebuffers.framebuffers.len() as u32)
            .command_pool(command_pool.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);
        let command_buffers = unsafe {
            core.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("failed to allocate command buffer")
        };

        let fences: Vec<vk::Fence> = command_buffers
            .iter()
            .map(|_| {
                let fence_create_info =
                    vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
                unsafe {
                    core.device
                        .create_fence(&fence_create_info, None)
                        .expect("failed to create fence")
                }
            })
            .collect();
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        let image_available_semaphore =
            unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
                .expect("failed to create semaphore");
        let render_finished_semaphore =
            unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
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
        core: &mut Core,
        framebuffer: &vk::Framebuffer,
        graphics_pipeline: &GraphicsPipeline,
        width: u32,
        height: u32,
        image_index: usize,
        uniform_buffers: &HashMap<String, UniformBuffer>,
        mesh_list: &[RenderCollectionMesh],
        clear_op: ClearOp,
    ) {
        let begin_info = vk::CommandBufferBeginInfo::builder();
        core.device
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
        core.device.cmd_begin_render_pass(
            self.command_buffers[image_index],
            &renderpass_info,
            vk::SubpassContents::INLINE,
        );
        core.device.cmd_bind_pipeline(
            self.command_buffers[image_index],
            vk::PipelineBindPoint::GRAPHICS,
            match clear_op {
                ClearOp::ClearColor => graphics_pipeline.clear_pipeline.graphics_pipeline,
                ClearOp::DoNotClear => graphics_pipeline.load_pipeline.graphics_pipeline,
            },
        );
        for mesh in mesh_list.iter() {
            core.device.cmd_bind_vertex_buffers(
                self.command_buffers[image_index],
                0,
                &[mesh.vertex_buffer.buffer],
                &[0],
            );
            core.device.cmd_bind_index_buffer(
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
            core.device.cmd_bind_descriptor_sets(
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
            core.device.cmd_push_constants(
                self.command_buffers[image_index],
                graphics_pipeline.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                matrix_slice,
            );
            core.device.cmd_draw_indexed(
                self.command_buffers[image_index],
                mesh.index_buffer.get_num_indicies() as u32,
                1,
                mesh.offsets.index_offset as u32,
                mesh.offsets.vertex_offset as i32,
                0,
            );
        }

        core.device
            .cmd_end_render_pass(self.command_buffers[image_index]);
        core.device
            .end_command_buffer(self.command_buffers[image_index])
            .expect("failed to create command buffer");
        core.device
            .reset_fences(&[self.fences[image_index as usize]])
            .expect("failed to reset fence");
    }
    pub unsafe fn render_frame(
        &mut self,
        core: &mut Core,
        framebuffer: &Framebuffer,
        graphics_pipeline: &GraphicsPipeline,
        width: u32,
        height: u32,
        uniform_buffers: &mut HashMap<String, UniformBuffer>,
        meshes: &RenderCollection,
    ) {
        let (image_index, _) = core
            .swapchain_loader
            .acquire_next_image(
                core.swapchain,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )
            .expect("failed to aquire image");
        let semaphores = self
            .semaphore_buffer
            .get_semaphores(core, meshes.num_uniforms_to_update());
        for (idx, ((uniform_name, mesh), semaphore)) in
            meshes.batches.iter().zip(semaphores).enumerate()
        {
            for (uniform_data, mesh) in mesh.iter() {
                core.device
                    .wait_for_fences(&[self.fences[image_index as usize]], true, u64::MAX)
                    .expect("failed to wait for fence");
                core.device
                    .reset_fences(&[self.fences[image_index as usize]])
                    .expect("failed to reset fence");

                uniform_buffers
                    .get_mut(uniform_name)
                    .expect(&format!("failed to find uniform \" {}\"", &uniform_name))
                    .update_uniform(core, image_index as usize, &uniform_data);
                self.build_renderpass(
                    core,
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

                core.device
                    .queue_submit(
                        core.present_queue,
                        &[submit_info],
                        self.fences[image_index as usize],
                    )
                    .expect("failed to submit queue");
            }
        }
        let wait_semaphores = [core.swapchain];
        let image_indices = [image_index];
        let present_wait_semaphore = [self.semaphore_buffer.render_finished_semaphore()];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&present_wait_semaphore)
            .swapchains(&wait_semaphores)
            .image_indices(&image_indices);
        core.swapchain_loader
            .queue_present(core.present_queue, &present_info)
            .expect("failed to present queue");
    }
    pub fn wait_idle(&mut self, core: &mut Core) {
        unsafe {
            core.device
                .wait_for_fences(&self.fences, true, 10000000)
                .expect("failed to wait for fence");
            core.device.device_wait_idle().expect("failed to wait idle");
        }
    }
    pub fn free(&mut self, core: &mut Core) {
        unsafe {
            core.device
                .wait_for_fences(&self.fences, true, 10000000)
                .expect("failed to wait for fence");
            core.device.device_wait_idle().expect("failed to wait idle");
            core.device
                .destroy_semaphore(self.render_finished_semaphore, None);
            core.device
                .destroy_semaphore(self.image_available_semaphore, None);
            self.semaphore_buffer.free(core);
            for fence in self.fences.iter() {
                core.device.destroy_fence(*fence, None);
            }
        }
    }
}
