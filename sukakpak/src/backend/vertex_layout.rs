use ash::vk;
use std::mem::size_of;
pub enum VertexComponent {
    Vec1F32,
    Vec2F32,
    Vec3F32,
    Vec4F32,
}
impl VertexComponent {
    pub fn size(&self) -> usize {
        match self {
            Self::Vec1F32 => size_of::<f32>(),
            Self::Vec2F32 => 2 * size_of::<f32>(),
            Self::Vec3F32 => 3 * size_of::<f32>(),
            Self::Vec4F32 => 4 * size_of::<f32>(),
        }
    }
}
impl From<&VertexComponent> for vk::Format {
    fn from(comp: &VertexComponent) -> Self {
        match comp {
            &VertexComponent::Vec1F32 => vk::Format::R32_SFLOAT,
            &VertexComponent::Vec2F32 => vk::Format::R32G32_SFLOAT,
            &VertexComponent::Vec3F32 => vk::Format::R32G32B32_SFLOAT,
            &VertexComponent::Vec4F32 => vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}
pub struct VertexLayout {
    pub components: Vec<VertexComponent>,
}
