use std::mem::size_of;
#[derive(Clone, Debug, PartialEq)]
pub enum VertexComponent {
    Vec1F32,
    Vec2F32,
    Vec3F32,
    Vec4F32,
}
impl VertexComponent {
    /// Gets number of components in vertex
    pub fn num_components(&self) -> usize {
        match self {
            Self::Vec1F32 => 1,
            Self::Vec2F32 => 2,
            Self::Vec3F32 => 3,
            Self::Vec4F32 => 4,
        }
    }
    /// Gets size in bytes of each component
    pub fn size(&self) -> usize {
        match self {
            Self::Vec1F32 => size_of::<f32>(),
            Self::Vec2F32 => 2 * size_of::<f32>(),
            Self::Vec3F32 => 3 * size_of::<f32>(),
            Self::Vec4F32 => 4 * size_of::<f32>(),
        }
    }
}
/// Layout of vertex in mesh, order reperesents `location` in
/// Webgl and Vulkan backends
#[derive(Clone, Debug, PartialEq)]
pub struct VertexLayout {
    pub components: Vec<VertexComponent>,
}
