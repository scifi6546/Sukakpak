use super::VertexComponent;
use ash::vk;

impl From<&VertexComponent> for vk::Format {
    fn from(comp: &VertexComponent) -> Self {
        match *comp {
            VertexComponent::Vec1F32 => vk::Format::R32_SFLOAT,
            VertexComponent::Vec2F32 => vk::Format::R32G32_SFLOAT,
            VertexComponent::Vec3F32 => vk::Format::R32G32B32_SFLOAT,
            VertexComponent::Vec4F32 => vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}
