use super::{
    CommandPool, Device, IndexBuffer, OffsetData, RenderMesh, Texture, TextureID, Vertex,
    VertexBuffer,
};

use generational_arena::{Arena, Index as ArenaIndex};
use nalgebra::Matrix4;
#[derive(Clone, Copy)]
pub struct MeshID {
    pub index: ArenaIndex,
}
pub struct Mesh {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    texture: TextureID,
}
//offset pointer to mesh

pub struct MeshOffset {
    pub mesh: MeshID,
    pub vertex_buffer_offset: usize,
    pub index_buffer_offset: usize,
    pub texture: TextureID,
}
#[derive(Clone, Copy)]
pub struct MeshOffsetID {
    pub index: ArenaIndex,
}
impl Mesh {
    pub fn new(
        device: &mut Device,
        command_pool: &mut CommandPool,
        texture: TextureID,
        verticies: Vec<Vertex>,
        indicies: Vec<u32>,
    ) -> Self {
        Self {
            vertex_buffer: VertexBuffer::new(device, verticies),
            index_buffer: IndexBuffer::new(device, command_pool, indicies),
            texture,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        self.vertex_buffer.free(device);
        self.index_buffer.free(device);
    }
    pub fn to_render_mesh<'a>(
        &'a self,
        uniform_data: *const std::ffi::c_void,
        texture_arena: &'a Arena<Texture>,
        offset: &MeshOffset,
    ) -> RenderMesh<'a, { std::mem::size_of::<Matrix4<f32>>() }> {
        RenderMesh {
            vertex_buffer: &self.vertex_buffer,
            index_buffer: &self.index_buffer,
            uniform_data,
            texture: texture_arena
                .get(self.texture.index)
                .expect("failed to get texture"),
            offsets: OffsetData {
                vertex_offset: offset.vertex_buffer_offset,
                index_offset: offset.index_buffer_offset,
            },
        }
    }
}
