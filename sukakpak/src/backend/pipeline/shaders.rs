use ash::vk;
use nalgebra::{Matrix4, Vector3};
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub struct PushConstantDesc {
    pub range: vk::PushConstantRange,
}
pub struct UniformDescription {
    pub size: usize,
    pub descriptor_set_layout_binding: vk::DescriptorSetLayoutBinding,
}
pub struct ShaderDescription {
    pub uniforms: HashMap<String, UniformDescription>,
    pub push_constants: HashMap<String, PushConstantDesc>,
    pub vertex_buffer_desc: VertexBufferDesc,
    pub vertex_shader_data: &'static [u8],
    pub fragment_shader_data: &'static [u8],
}

pub struct VertexBufferDesc {
    pub binding_description: vk::VertexInputBindingDescription,
    pub attributes: &'static [vk::VertexInputAttributeDescription],
}
pub fn push_shader() -> ShaderDescription {
    ShaderDescription {
        uniforms: HashMap::new(),
        push_constants: [(
            "view".to_string(),
            PushConstantDesc {
                range: vk::PushConstantRange {
                    offset: 0,
                    size: std::mem::size_of::<Matrix4<f32>>() as u32,
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                },
            },
        )]
        .iter()
        .cloned()
        .collect(),
        vertex_buffer_desc: VertexBufferDesc {
            binding_description: vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<f32>() as u32 * 5,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            attributes: &[
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: 0,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: std::mem::size_of::<Vector3<f32>>() as u32,
                },
            ],
        },
        fragment_shader_data: include_bytes!("../../../shaders/push.frag.spv"),
        vertex_shader_data: include_bytes!("../../../shaders/push.vert.spv"),
    }
}
