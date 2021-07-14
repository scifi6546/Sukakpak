use super::DescriptorDesc;
use ash::vk;
use ass_lib::{AssembledSpirv, ScalarType, ShaderStage, Type};
use nalgebra::{Matrix4, Vector3};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug)]
pub struct PushConstantDesc {
    pub range: vk::PushConstantRange,
}
#[derive(Clone, Debug)]
pub struct ShaderDescription {
    pub push_constants: Vec<PushConstantDesc>,
    pub vertex_buffer_desc: VertexBufferDesc,
    pub vertex_shader_data: Vec<u8>,
    pub fragment_shader_data: Vec<u8>,
    pub textures: HashMap<String, DescriptorDesc>,
}

#[derive(Clone, Debug)]
pub struct VertexBufferDesc {
    pub binding_description: vk::VertexInputBindingDescription,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
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
        let binding_description = vk::VertexInputBindingDescription {
            binding: spv.vertex_shader.vertex_input_binding,
            stride: spv
                .vertex_shader
                .data_in
                .iter()
                .map(|(data_type, _location)| data_type.size())
                .sum(),
            input_rate: vk::VertexInputRate::VERTEX,
        };
        let mut offset = 0;
        let attributes = spv
            .vertex_shader
            .data_in
            .iter()
            .map(|(data_type, location)| {
                let current_offset = offset;
                offset += data_type.size();
                vk::VertexInputAttributeDescription {
                    binding: spv.vertex_shader.vertex_input_binding,
                    location: location.location,
                    offset: current_offset,
                    format: match data_type {
                        Type::Scalar(ty) => match ty {
                            ScalarType::F32 => vk::Format::R32_SFLOAT,
                            ScalarType::F64 => vk::Format::R64_SFLOAT,
                        },
                        Type::Vec2(ty) => match ty {
                            ScalarType::F32 => vk::Format::R32G32_SFLOAT,
                            ScalarType::F64 => vk::Format::R64G64_SFLOAT,
                        },
                        Type::Vec3(ty) => match ty {
                            ScalarType::F32 => vk::Format::R32G32B32_SFLOAT,
                            ScalarType::F64 => todo!(),
                        },
                        Type::Vec4(ty) => match ty {
                            ScalarType::F32 => vk::Format::R32G32B32A32_SFLOAT,
                            ScalarType::F64 => todo!(),
                        },
                        _ => todo!(),
                    },
                }
            })
            .collect();
        let vertex_buffer_desc = VertexBufferDesc {
            binding_description,
            attributes,
        };
        let textures = spv
            .textures
            .iter()
            .map(|(name, tex)| {
                (
                    name.clone(),
                    DescriptorDesc {
                        layout_binding: *vk::DescriptorSetLayoutBinding::builder()
                            .binding(tex.binding)
                            .descriptor_count(0)
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                    },
                )
            })
            .collect();
        Self {
            push_constants,
            vertex_buffer_desc,
            vertex_shader_data: spv
                .vertex_shader
                .data
                .iter()
                .map(|int| int.to_ne_bytes())
                .flatten()
                .collect(),
            fragment_shader_data: spv
                .fragment_shader
                .data
                .iter()
                .map(|int| int.to_ne_bytes())
                .flatten()
                .collect(),
            textures,
        }
    }
}
pub fn push_shader() -> ShaderDescription {
    ShaderDescription {
        textures: [(
            "mesh_texture".to_string(),
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
            attributes: vec![
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
                vk::VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: 0,
                },
            ],
        },
        fragment_shader_data: include_bytes!("../../../shaders/push.frag.spv")
            .iter()
            .copied()
            .collect(),
        vertex_shader_data: include_bytes!("../../../shaders/push.vert.spv")
            .iter()
            .copied()
            .collect(),
    }
}
pub fn alt_shader() -> ShaderDescription {
    ShaderDescription {
        textures: [(
            "mesh_texture".to_string(),
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
            attributes: vec![
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
        fragment_shader_data: include_bytes!("../../../shaders/alt.frag.spv")
            .iter()
            .copied()
            .collect(),
        vertex_shader_data: include_bytes!("../../../shaders/alt.vert.spv")
            .iter()
            .copied()
            .collect(),
    }
}
