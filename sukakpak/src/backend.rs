use anyhow::Result;

use nalgebra::Vector2;
mod render_core;
mod vertex_buffer;
use super::Mesh;
use generational_arena::{Arena, Index as ArenaIndex};
use render_core::Core;
use vertex_buffer::{VertexBufferAllocation, VertexBufferPool};
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
    vertex_buffer_pool: VertexBufferPool,
    core: Core,
}
pub struct VertexBufferID {
    buffer_index: ArenaIndex,
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
        let core = Core::new(&window, &create_info)?;
        let vertex_buffer_pool = VertexBufferPool::new(&core);

        Ok(Self {
            window,
            core,
            vertex_buffer_pool,
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
                .insert(self.vertex_buffer_pool.allocate_buffer(
                    &mut self.core,
                    mesh,
                    vertex_layout,
                )?),
        })
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        unsafe {
            for (_idx, mesh) in self.vertex_buffers.drain() {
                mesh.free(&mut self.core, &mut self.vertex_buffer_pool);
            }
            self.vertex_buffer_pool.free();
            self.core.free();
        }
    }
}
