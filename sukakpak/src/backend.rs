use anyhow::Result;

use nalgebra::Vector2;
mod command_pool;
mod depth_buffer;
mod framebuffer;
mod present_image;
mod render_core;
mod renderpass;
mod resource_pool;
use command_pool::CommandPool;
use depth_buffer::DepthBuffer;
use framebuffer::Framebuffer;
use generational_arena::{Arena, Index as ArenaIndex};
use pipeline::GraphicsPipeline;
use present_image::PresentImage;
use render_core::Core;
mod pipeline;
use resource_pool::{
    IndexBufferAllocation, ResourcePool, TextureAllocation, UniformAllocation,
    VertexBufferAllocation,
};

pub struct BackendCreateInfo {
    pub default_size: Vector2<u32>,
    pub name: String,
}
//layout of vertex
pub enum VertexLayout {
    XYZ_F32, //xyz vector with floating point components
}
pub struct Backend {
    window: winit::window::Window,
    vertex_buffers: Arena<VertexBufferAllocation>,
    index_buffers: Arena<IndexBufferAllocation>,
    command_pool: CommandPool,
    resource_pool: ResourcePool,
    core: Core,
}
pub struct VertexBufferID {
    buffer_index: ArenaIndex,
}
pub struct IndexBufferID {
    buffer_index: ArenaIndex,
}
pub struct MeshID {
    pub verticies: VertexBufferID,
    pub indicies: IndexBufferID,
}
impl Backend {
    pub fn new(
        create_info: BackendCreateInfo,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Result<Self> {
        let window = winit::window::WindowBuilder::new()
            .with_title(create_info.name.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                create_info.default_size.x,
                create_info.default_size.y,
            ))
            .build(&event_loop)?;
        let mut core = Core::new(&window, &create_info)?;
        let resource_pool = ResourcePool::new(&core)?;
        let command_pool = CommandPool::new(&mut core);

        Ok(Self {
            window,
            core,
            resource_pool,
            command_pool,
            index_buffers: Arena::new(),
            vertex_buffers: Arena::new(),
        })
    }
    pub fn allocate_verticies(
        &mut self,
        mesh: Vec<u8>,
        vertex_layout: VertexLayout,
    ) -> Result<VertexBufferID> {
        Ok(VertexBufferID {
            buffer_index: self
                .vertex_buffers
                .insert(self.resource_pool.allocate_vertex_buffer(
                    &mut self.core,
                    mesh,
                    vertex_layout,
                )?),
        })
    }
    pub fn allocate_indicies(&mut self, indicies: Vec<u32>) -> Result<IndexBufferID> {
        Ok(IndexBufferID {
            buffer_index: self
                .index_buffers
                .insert(self.resource_pool.allocate_index_buffer(
                    &mut self.core,
                    &mut self.command_pool,
                    indicies,
                )?),
        })
    }
    pub fn allocate_texture(&mut self, texture: &std::path::Path) {
        todo!()
    }
    pub fn draw_mesh(&mut self, mesh: &MeshID) -> Result<()> {
        todo!()
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        unsafe {
            for (_idx, mesh) in self.vertex_buffers.drain() {
                mesh.free(&mut self.core, &mut self.resource_pool);
            }
            for (_idx, buff) in self.index_buffers.drain() {
                buff.free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free buffer");
            }
            self.command_pool.free(&mut self.core);
            self.resource_pool.free();
            self.core.free();
        }
    }
}
