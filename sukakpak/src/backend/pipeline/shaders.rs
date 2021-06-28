use super::{DescriptorDesc, DescriptorName};
use ash::vk;
use ass_lib::{AssembledSpirv, ShaderStage};
use nalgebra::{Matrix4, Vector3};
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub struct PushConstantDesc {
    pub range: vk::PushConstantRange,
}
pub struct ShaderDescription {
    pub push_constants: Vec<PushConstantDesc>,
    pub vertex_buffer_desc: VertexBufferDesc,
    pub vertex_shader_data: &'static [u8],
    pub fragment_shader_data: &'static [u8],
    pub textures: HashMap<DescriptorName, DescriptorDesc>,
}

pub struct VertexBufferDesc {
    pub binding_description: vk::VertexInputBindingDescription,
    pub attributes: &'static [vk::VertexInputAttributeDescription],
}
impl From<AssembledSpirv> for ShaderDescription {
    fn from(spv: AssembledSpirv) -> Self {
        let push_constants = spv
            .push_constants
            .iter()
            .map(|constant| PushConstantDesc {
                range: vk::PushConstantRange {
                    offset: constant.offset,
                    size: constant.size,
                    stage_flags: match constant.stage {
                        ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
                        ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                    },
                },
            })
            .collect();
        Self {
            push_constants,
            vertex_buffer_desc: todo!(),
            vertex_shader_data: todo!(),
            fragment_shader_data: todo!(),
            textures: todo!(),
        }
    }
}
pub fn push_shader() -> ShaderDescription {
    ShaderDescription {
        textures: [(
            DescriptorName::MeshTexture,
            DescriptorDesc {
                layout_binding: *vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            },
        )]
        .iter()
        .cloned()
        .collect(),
        push_constants: vec![PushConstantDesc {
            range: vk::PushConstantRange {
                offset: 0,
                size: std::mem::size_of::<Matrix4<f32>>() as u32,
                stage_flags: vk::ShaderStageFlags::VERTEX,
            },
        }],
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
