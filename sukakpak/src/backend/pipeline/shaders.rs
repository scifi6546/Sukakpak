use std::collections::HashMap;
pub struct UniformDescription {
    name: &'static str,
    binding: u32,
    size: u32,
}
pub struct PushConstantDescription {
    name: &'static str,
    binding: u32,
    size: u32,
}
pub struct ShaderDescription {
    pub uniforms: &'static [UniformDescription],
    pub push_constants: &'static [PushConstantDescription],
    pub vertex_shader_data: &'static [u8],
    pub fragment_shader_data: &'static [u8],
}

const MAIN_SHADER: ShaderDescription = ShaderDescription {
    uniforms: &[],
    push_constants: &[],
    fragment_shader_data: include_bytes!("../../../shaders/main.frag.spv"),
    vertex_shader_data: include_bytes!("../../../shaders/main.vert.spv"),
};
