use super::DescriptorDesc;
use ash::vk;

use nalgebra::{Matrix4, Vector2, Vector3};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug)]
pub struct PushConstantDesc {
    pub range: vk::PushConstantRange,
}
/// Descriptor set for layout.
#[derive(Clone, Copy, Debug)]
pub struct TextureDescriptorLayout {
    pub image_layout_binding: vk::DescriptorSetLayoutBinding,
    pub sampler_layout_binding: vk::DescriptorSetLayoutBinding,
}
#[derive(Clone, Debug)]
pub struct ShaderDescription {
    pub push_constants: Vec<PushConstantDesc>,
    pub vertex_buffer_desc: VertexBufferDesc,
    pub vertex_shader_data: Vec<u8>,
    pub fragment_shader_data: Vec<u8>,
    pub textures: HashMap<String, TextureDescriptorLayout>,
    /// Name of vertex shader entrypoint, if v1 shader is "main"
    pub vertex_entrypoint: String,
    /// Name of fragment shader entrypoint, if v1 shader is "main"
    pub fragment_entrypoint: String,
}

#[derive(Clone, Debug)]
pub struct VertexBufferDesc {
    pub binding_description: vk::VertexInputBindingDescription,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
}
/// a barebones shader that just does test corrections, todo: make it simple with no change to colors
pub fn basic_shader() -> ShaderDescription {
    let shader =
        ass_vk::Shader::from_json_str(include_str!("../../../../shaders/v2/v2_test.ass_spv"))
            .ok()
            .unwrap();
    shader.into()
}
impl From<ass_vk::Shader> for ShaderDescription {
    fn from(shader: ass_vk::Shader) -> Self {
        let push_constants = vec![PushConstantDesc {
            range: vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                offset: 0,
                size: shader.push_constant.size(),
            },
        }];
        let mut attributes = vec![];
        let mut offset = 0;
        for input in shader.vertex_input.fields.iter() {
            attributes.push(vk::VertexInputAttributeDescription {
                location: input.location,
                binding: shader.vertex_input.binding,
                format: match input.ty {
                    ass_lib::ShaderType::Mat4x4(_) => {
                        panic!("matrix 4x4 not avalible as vertex input")
                    }
                    ass_lib::ShaderType::Vec4(s) => match s {
                        ass_lib::Scalar::F32 => vk::Format::R32G32B32A32_SFLOAT,
                        ass_lib::Scalar::U32 => vk::Format::R32G32B32A32_UINT,
                    },
                    ass_lib::ShaderType::Vec3(s) => match s {
                        ass_lib::Scalar::F32 => vk::Format::R32G32B32_SFLOAT,
                        ass_lib::Scalar::U32 => vk::Format::R32G32B32_UINT,
                    },
                    ass_lib::ShaderType::Vec2(s) => match s {
                        ass_lib::Scalar::F32 => vk::Format::R32G32_SFLOAT,
                        ass_lib::Scalar::U32 => vk::Format::R32G32_UINT,
                    },
                    ass_lib::ShaderType::Scalar(s) => match s {
                        ass_lib::Scalar::F32 => vk::Format::R32_SFLOAT,
                        ass_lib::Scalar::U32 => vk::Format::R32_UINT,
                    },
                    ass_lib::ShaderType::Struct(_) => panic!("struct invalid as vertex input"),
                },
                offset,
            });
            offset += input.size();
        }
        ShaderDescription {
            push_constants,
            vertex_buffer_desc: VertexBufferDesc {
                binding_description: vk::VertexInputBindingDescription {
                    binding: shader.vertex_input.binding,
                    stride: shader
                        .vertex_input
                        .fields
                        .iter()
                        .map(|f| f.size())
                        .fold(0, |acc, x| acc + x),
                    input_rate: vk::VertexInputRate::VERTEX,
                },
                attributes,
            },
            vertex_shader_data: shader
                .vertex_spirv_data
                .iter()
                .flat_map(|u| u.to_ne_bytes())
                .collect(),
            fragment_shader_data: shader
                .fragment_spirv_data
                .iter()
                .flat_map(|u| u.to_ne_bytes())
                .collect(),
            fragment_entrypoint: shader.fragment_entrypoint,
            vertex_entrypoint: shader.vertex_entrypoint,
            textures: shader
                .textures
                .iter()
                .zip(shader.samplers.iter())
                .map(|(tex, sampler)| {
                    (
                        tex.name.clone(),
                        TextureDescriptorLayout {
                            image_layout_binding: *vk::DescriptorSetLayoutBinding::builder()
                                .binding(tex.binding)
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                                .stage_flags(
                                    vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
                                ),
                            sampler_layout_binding: *vk::DescriptorSetLayoutBinding::builder()
                                .binding(sampler.binding)
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::SAMPLER)
                                .stage_flags(
                                    vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
                                ),
                        },
                    )
                })
                .collect(),
        }
    }
}
