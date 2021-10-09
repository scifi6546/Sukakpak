use ass_lib_v2::{anyhow::Result, ShaderIR};
fn run() -> Result<()> {
    let ir = ShaderIR::compile_from_disk("test_shader")?;
    let vulkan = ass_lib_v2::vk::Shader::from_ir(ir)?;
    Ok(())
}
fn main() {
    run().expect("failed to compile shader");
}
