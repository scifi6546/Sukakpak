use super::{CommandPool, Device, TextureID, Vertex, VertexBuffer};
pub struct Mesh {
    vertex_buffer: VertexBuffer,
    texture: TextureID,
}
impl Mesh {
    pub fn new(device: &mut Device, texture: TextureID, verticies: Vec<Vertex>) -> Self {
        Self {
            vertex_buffer: VertexBuffer::new(device, verticies),
            texture,
        }
    }
}
