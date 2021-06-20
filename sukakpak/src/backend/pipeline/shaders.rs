use ash::vk;
use nalgebra::{Matrix4, Vector3};
use phf::phf_map;
#[derive(Clone, Copy)]
pub struct PushConstantDesc {
    pub range: vk::PushConstantRange,
}
pub struct UniformDescription {
    pub size: usize,
    pub descriptor_set_layout_binding: vk::DescriptorSetLayoutBinding,
}
pub struct ShaderDescription {
    pub uniforms: phf::Map<&'static str, UniformDescription>,
    pub push_constants: phf::Map<&'static str, PushConstantDesc>,
    pub vertex_buffer_desc: VertexBufferDesc,
    pub vertex_shader_data: &'static [u8],
    pub fragment_shader_data: &'static [u8],
}

pub struct VertexBufferDesc {
    pub binding_description: vk::VertexInputBindingDescription,
    pub attributes: &'static [vk::VertexInputAttributeDescription],
}
pub const MAIN_SHADER: ShaderDescription = ShaderDescription {
    uniforms: phf_map! {
            "view"=>UniformDescription {
                size: std::mem::size_of::<Matrix4<f32>>(),
                descriptor_set_layout_binding: vk::DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    p_immutable_samplers: std::ptr::null(),
                },
        }
    },
    push_constants: phf_map! {},
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
    fragment_shader_data: include_bytes!("../../../shaders/main.frag.spv"),
    vertex_shader_data: include_bytes!("../../../shaders/main.vert.spv"),
};
pub const PUSH_SHADER: ShaderDescription = ShaderDescription {
    uniforms: phf_map! {},
    push_constants: phf_map! {
        "view" =>PushConstantDesc{
            range: vk::PushConstantRange{
                offset:0,
                size: std::mem::size_of::<Matrix4<f32>>() as u32,
                stage_flags: vk::ShaderStageFlags::VERTEX
            }
        }
    },
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
};
