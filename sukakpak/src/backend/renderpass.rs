use super::{
    CommandPool, Core, Framebuffer, GraphicsPipeline, IndexBufferAllocation, Texture,
    UniformBuffer, VertexBuffer,
};
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use nalgebra::{Matrix4, Vector2};
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
    image_index: Option<u32>,
    first_in_frame: bool,
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
            image_index: None,
            first_in_frame: false,
        }
    }
    pub fn draw_mesh(
        &mut self,
        core: &mut Core,
        graphics_pipeline: &GraphicsPipeline,
        mesh: RenderMesh,
    ) {
        if let Some(image_index) = self.image_index {
            unsafe {
                core.device.cmd_bind_vertex_buffers(
                    self.command_buffers[image_index as usize],
                    0,
                    &[mesh.vertex_buffer],
                    &[0],
                );
                core.device.cmd_bind_index_buffer(
                    self.command_buffers[image_index as usize],
                    mesh.index_buffer.buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                core.device.cmd_bind_descriptor_sets(
                    self.command_buffers[image_index as usize],
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline.pipeline_layout,
                    0,
                    &[
                        todo!("figure out uniform buffers"),
                        todo!("figure out texture descriptor sets"),
                    ],
                    &[],
                );
                core.device.cmd_draw_indexed(
                    self.command_buffers[image_index as usize],
                    mesh.index_buffer.num_indices() as u32,
                    1,
                    0,
                    0,
                    0,
                );
            }
        } else {
            self.acquire_next_image(core);
            self.begin_renderpass(core, ClearOp::DoNotClear);
            self.draw_mesh(core, mesh);
        }
    }
    pub fn begin_renderpass(
        &mut self,
        core: &mut Core,
        graphics_pipeline: &GraphicsPipeline,
        framebuffer: &vk::Framebuffer,
        clear_op: ClearOp,
        dimensions: Vector2<u32>,
    ) {
        if let Some(image_index) = self.image_index {
            unsafe {
                core.device.begin_command_buffer(
                    self.command_buffers[image_index as usize],
                    &vk::CommandBufferBeginInfo::builder(),
                );
                let renderpass_info = vk::RenderPassBeginInfo::builder()
                    .render_pass(match clear_op {
                        ClearOp::ClearColor => graphics_pipeline.clear_pipeline.renderpass,
                        ClearOp::DoNotClear => graphics_pipeline.load_pipeline.renderpass,
                    })
                    .framebuffer(*framebuffer)
                    .render_area(vk::Rect2D {
                        extent: vk::Extent2D {
                            width: dimensions.x,
                            height: dimensions.y,
                        },
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
                    self.command_buffers[image_index as usize],
                    &renderpass_info,
                    vk::SubpassContents::INLINE,
                );
                core.device.cmd_bind_pipeline(
                    self.command_buffers[image_index as usize],
                    vk::PipelineBindPoint::GRAPHICS,
                    match clear_op {
                        ClearOp::ClearColor => graphics_pipeline.clear_pipeline.graphics_pipeline,
                        ClearOp::DoNotClear => graphics_pipeline.load_pipeline.graphics_pipeline,
                    },
                );
            }
        } else {
            self.acquire_next_image(core);
            self.begin_renderpass(core, clear_op);
        }
    }
    pub fn submit_draw() {
        todo!()
    }
    pub fn upload_uniform() {
        todo!()
    }
    //aquires new image index and populates self.image_index
    pub fn acquire_next_image(&mut self, core: &mut Core) -> Result<()> {
        let (image_index, _) = core.swapchain_loader.acquire_next_image(
            core.swapchain,
            u64::MAX,
            self.image_available_semaphore,
            vk::Fence::null(),
        )?;
        self.image_index = Some(image_index);
        Ok(())
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
