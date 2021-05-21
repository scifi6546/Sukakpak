use super::{Device, Vertex, VertexBuffer};
pub struct Mesh {
    vertex_buffer: VertexBuffer,
}
impl Mesh {
    pub fn new(device: &mut Device, verticies: Vec<Vertex>) -> Self {
        Self {
            vertex_buffer: VertexBuffer::new(device, verticies),
        }
    }
}
