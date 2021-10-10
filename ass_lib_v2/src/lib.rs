mod shader_type;
pub use anyhow;
use anyhow::{bail, Result};
use naga::front::wgsl;
use serde::Deserialize;
pub use shader_type::{Scalar, ShaderType};
use std::{fs::File, io::Read, path::Path};
const VERTEX_SHADER_MAIN: &'static str = "vs_main";
const FRAGMENT_SHADER_MAIN: &'static str = "fs_main";

#[derive(Deserialize)]
pub struct Project {
    /// Path relative to project root specififying location of vertex shader
    pub shader_path: String,
}
/// Intermediate represention of shader. Used to convert to production
/// shading langages such as SPIR-V for Vulkan or glsl for webgl2.
pub struct ShaderIR {
    module: naga::Module,
    info: naga::valid::ModuleInfo,
}
impl ShaderIR {
    pub fn compile_from_disk<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path: &Path = path.as_ref();
        let file = File::open(path.join("project.json"))?;
        let project_data: Project = serde_json::from_reader(file)?;

        let mut shader_string = String::new();
        let result = File::open(path.join(project_data.shader_path.clone()));
        if result.is_err() {
            let err_kind = result.err().unwrap().kind();
            match err_kind {
                std::io::ErrorKind::NotFound => {
                    bail!("shader file: {} not found", project_data.shader_path)
                }
                _ => bail!(
                    "other error reading shader file: {}",
                    project_data.shader_path
                ),
            }
        }
        result.ok().unwrap().read_to_string(&mut shader_string)?;

        let module = wgsl::parse_str(&shader_string)?;
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::PUSH_CONSTANT,
        );

        let info = validator.validate(&module)?;
        Ok(Self { module, info })
    }
}

pub mod vk;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
