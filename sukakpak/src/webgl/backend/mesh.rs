use super::DrawableTexture;
use web_sys::{WebGlBuffer, WebGlVertexArrayObject as VAO};
#[derive(Debug, Clone, PartialEq, Eq)]
/// Describes mesh data for drawing
pub struct Mesh {
    pub vao: VAO,
    pub buffer: WebGlBuffer,
    pub texture: DrawableTexture,
    /// number of verticies to draw
    pub num_vertices: usize,
}
impl Mesh {
    /// Gets the number of vertices of the mesh
    pub fn get_num_vertices(&self) -> usize {
        self.num_vertices
    }
}
