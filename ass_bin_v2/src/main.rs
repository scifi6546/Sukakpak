use ass_lib_v2::{anyhow::Result, ShaderIR};
fn run() -> Result<()> {
    let ir = ShaderIR::compile_from_disk("test_shader")?;
    let vulkan = ass_lib_v2::vk::Shader::from_ir(ir)?;
    println!("\n\n{}", vulkan.to_json_string()?);
    vulkan.write_to_disk("test")?;
    let v2 = ass_lib_v2::vk::Shader::read_from_disk("test")?;
    println!("\n\nnew shader: \n\n{}", v2.to_json_string()?);

    Ok(())
}
fn main() {
    run().expect("failed to compile shader");
}
