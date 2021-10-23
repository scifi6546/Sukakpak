use super::DrawableTexture;
use web_sys::{WebGlBuffer, WebGlVertexArrayObject as VAO};
#[derive(Debug, Clone, PartialEq, Eq)]
/// Describes mesh data for drawing
pub struct Mesh {
    pub vao: VAO,
    pub buffer: WebGlBuffer,
    pub texture: DrawableTexture,
}
