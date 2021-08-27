use super::{
    CommandPool, Core, FrameBufferTarget, Framebuffer, IndexBufferAllocation,
    VertexBufferAllocation,
};
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use generational_arena::Index as ArenaIndex;
use nalgebra::Vector2;
use std::collections::HashSet;
mod semaphore_buffer;
use free_list::FreeList;
use semaphore_buffer::SemaphoreBuffer;
pub enum ClearOp {
    ClearColor,
    DoNotClear,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
/// Contains ids of resoures used by mesh
pub struct RenderMeshIds {
    pub vertex_buffer_id: ArenaIndex,
    pub index_buffer_id: ArenaIndex,
}
pub struct RenderMesh<'a> {
    pub ids: RenderMeshIds,
    //pub uniform_data: HashMap<String, &'a [u8]>,
    pub push: Vec<u8>,
    pub vertex_buffer: &'a VertexBufferAllocation,
    pub index_buffer: &'a IndexBufferAllocation,
}

#[derive(Clone, Copy)]
// Offset of mesh to draw
pub struct OffsetData {
    /// offset in `std::mem::size_of::<f32>()*3*indicies`
    pub vertex_offset: usize,
    /// offset in `std::mem::size_of::<u32>()*1*indicies`
    pub index_offset: usize,
}
/// Keeps track of data used in renderpass
#[derive(Default, Debug)]
struct RenderpassGarbageCollector {
    mesh_freelist: FreeList<RenderMeshIds>,
}
impl RenderpassGarbageCollector {
    /// Marks data as used in given renderpass
    pub fn push(&mut self, id: RenderMeshIds, renderpass_id: u32) {
        self.mesh_freelist.push(id, renderpass_id);
    }
    /// marks data as to free. Data is only allowed to be released once renderpasse that use the
    /// resource are done.
    pub fn try_free(&mut self, id: RenderMeshIds) {
        self.mesh_freelist.try_free(id);
    }
    /// Marks a renderpass as done and returns all meshes that are no longer in use
    pub fn finish_renderpass(&mut self, renderpass_id: u32) -> HashSet<RenderMeshIds> {
        self.mesh_freelist.finish_renderpass(renderpass_id)
    }
}
pub struct FreedMeshes {
    pub meshes: HashSet<RenderMeshIds>,
}
pub struct RenderPass {
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
    semaphore_buffer: SemaphoreBuffer,
    image_available_semaphore: [vk::Semaphore; 1],
    garbage_collector: RenderpassGarbageCollector,
    image_index: Option<u32>,
}
impl RenderPass {
    pub fn new(
        core: &mut Core,
        command_pool: &CommandPool,
        framebuffer_target: &FrameBufferTarget,
    ) -> Self {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(framebuffer_target.framebuffers.len() as u32)
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
            [
                unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
                    .expect("failed to create semaphore"),
            ];

        let semaphore_buffer = SemaphoreBuffer::new(image_available_semaphore[0]);
        Self {
            command_buffers,
            fences,
            semaphore_buffer,
            image_available_semaphore,
            garbage_collector: Default::default(),
            image_index: None,
        }
    }
    pub fn draw_mesh(
        &mut self,
        core: &mut Core,
        framebuffer: &Framebuffer,
        descriptor_sets: &[vk::DescriptorSet],
        screen_dimensions: Vector2<u32>,
        mesh: RenderMesh,
    ) -> Result<()> {
        if let Some(image_index) = self.image_index {
            self.garbage_collector.push(mesh.ids, image_index);
            unsafe {
                core.device.cmd_bind_vertex_buffers(
                    self.command_buffers[image_index as usize],
                    0,
                    &[mesh.vertex_buffer.buffer],
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
                    framebuffer.pipeline.pipeline_layout,
                    0,
                    descriptor_sets,
                    &[],
                );
                if !mesh.push.is_empty() {
                    core.device.cmd_push_constants(
                        self.command_buffers[image_index as usize],
                        framebuffer.pipeline.pipeline_layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        &mesh.push,
                    );
                }

                core.device.cmd_draw_indexed(
                    self.command_buffers[image_index as usize],
                    mesh.index_buffer.num_indices() as u32,
                    1,
                    0,
                    0,
                    0,
                );
            }
            Ok(())
        } else {
            self.acquire_next_image(core)?;
            self.begin_renderpass(core, framebuffer, ClearOp::DoNotClear)?;
            self.draw_mesh(core, framebuffer, descriptor_sets, screen_dimensions, mesh)?;
            Ok(())
        }
    }
    /// begins rendering a frame, builds renderpass with selected frame
    pub unsafe fn begin_frame(&mut self, core: &mut Core, framebuffer: &Framebuffer) -> Result<()> {
        self.acquire_next_image(core)?;
        let image_index = self.image_index.unwrap();
        core.device
            .wait_for_fences(&[self.fences[image_index as usize]], true, u64::MAX)?;
        core.device.begin_command_buffer(
            self.command_buffers[image_index as usize],
            &vk::CommandBufferBeginInfo::builder(),
        )?;
        self.begin_renderpass(core, framebuffer, ClearOp::ClearColor)
    }

    pub fn begin_renderpass(
        &mut self,
        core: &mut Core,
        framebuffer: &Framebuffer,
        clear_op: ClearOp,
    ) -> Result<()> {
        let image_index = self
            .image_index
            .expect("invalid usage frame should be started with begin frame");
        unsafe {
            let renderpass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(match clear_op {
                    ClearOp::ClearColor => framebuffer.pipeline.clear_pipeline.renderpass,
                    ClearOp::DoNotClear => framebuffer.pipeline.load_pipeline.renderpass,
                })
                .framebuffer(framebuffer.framebuffer_target.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    extent: vk::Extent2D {
                        width: framebuffer.resolution.x,
                        height: framebuffer.resolution.y,
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
                    ClearOp::ClearColor => framebuffer.pipeline.clear_pipeline.graphics_pipeline,
                    ClearOp::DoNotClear => framebuffer.pipeline.load_pipeline.graphics_pipeline,
                },
            );
            Ok(())
        }
    }
    pub unsafe fn end_renderpass(&mut self, core: &mut Core) -> Result<()> {
        if let Some(image_index) = self.image_index {
            core.device
                .cmd_end_render_pass(self.command_buffers[image_index as usize]);
            Ok(())
        } else {
            panic!("renderpass should be started first")
        }
    }
    #[must_use]
    pub fn submit_draw(&mut self, core: &mut Core) -> Result<FreedMeshes> {
        if let Some(image_index) = self.image_index {
            unsafe {
                core.device
                    .cmd_end_render_pass(self.command_buffers[image_index as usize]);
                core.device
                    .end_command_buffer(self.command_buffers[image_index as usize])?;
                core.device.wait_for_fences(
                    &[self.fences[image_index as usize]],
                    true,
                    u64::MAX,
                )?;
            }

            unsafe {
                core.device
                    .reset_fences(&[self.fences[image_index as usize]])?;
                let submit_semaphore = self.semaphore_buffer.get_semaphore(core)?;
                let command_buffers = [self.command_buffers[image_index as usize]];
                let signal_semaphores = [submit_semaphore.finished_semaphore];

                let submit_info = *vk::SubmitInfo::builder()
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .command_buffers(&command_buffers)
                    .signal_semaphores(&signal_semaphores)
                    .wait_semaphores(&submit_semaphore.start_semaphore);
                core.device.queue_submit(
                    core.present_queue,
                    &[submit_info],
                    self.fences[image_index as usize],
                )?;
            }
            println!(
                "before freeing index: {}\n{:#?}",
                image_index, self.garbage_collector
            );
            let meshes = self.garbage_collector.finish_renderpass(image_index);
            println!(
                "after freeing index: {}\n{:#?}",
                image_index, self.garbage_collector
            );
            Ok(FreedMeshes { meshes })
        } else {
            self.acquire_next_image(core)?;
            self.submit_draw(core)
        }
    }
    /// Marks a mesh for freeing but it is only freed once it is unused by inprogress renderpasses
    pub fn free_mesh(&mut self, mesh: RenderMeshIds) {
        self.garbage_collector.try_free(mesh)
    }
    pub fn swap_framebuffer(&mut self, core: &mut Core) -> std::result::Result<(), vk::Result> {
        if let Some(image_index) = self.image_index {
            let indices = [image_index];
            let swapchain = [core.swapchain];
            let wait_semaphore = [self.semaphore_buffer.last_semaphore()];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphore)
                .swapchains(&swapchain)
                .image_indices(&indices);
            unsafe {
                core.swapchain_loader
                    .queue_present(core.present_queue, &present_info)?;
            }
            self.semaphore_buffer.reset();
            self.image_index = None;
            Ok(())
        } else {
            self.acquire_next_image(core)?;
            self.swap_framebuffer(core)
        }
    }
    //aquires new image index and populates self.image_index
    pub fn acquire_next_image(&mut self, core: &mut Core) -> std::result::Result<(), vk::Result> {
        let (image_index, _) = unsafe {
            core.swapchain_loader.acquire_next_image(
                core.swapchain,
                u64::MAX,
                self.semaphore_buffer.first_semaphore(),
                vk::Fence::null(),
            )
        }?;
        self.image_index = Some(image_index);
        Ok(())
    }
    pub fn get_image_index(&mut self, core: &mut Core) -> Result<usize> {
        if let Some(idx) = self.image_index {
            Ok(idx as usize)
        } else {
            self.acquire_next_image(core)?;
            self.get_image_index(core)
        }
    }
    pub fn wait_idle(&mut self, core: &mut Core) {
        unsafe {
            core.device
                .wait_for_fences(&self.fences, true, u64::MAX)
                .expect("failed to wait for fence");
            core.device.device_wait_idle().expect("failed to wait idle");
        }
    }
    pub fn free(&mut self, core: &mut Core) {
        self.wait_idle(core);
        unsafe {
            core.device
                .wait_for_fences(&self.fences, true, 10000000)
                .expect("failed to wait for fence");
            core.device.device_wait_idle().expect("failed to wait idle");
            core.device
                .destroy_semaphore(self.image_available_semaphore[0], None);
            self.semaphore_buffer.free(core);
            for fence in self.fences.iter() {
                core.device.destroy_fence(*fence, None);
            }
        }
    }
}
