mod shader_type;
pub use anyhow;
use anyhow::{bail, Result};
use naga::front::wgsl;
use serde::Deserialize;
pub use shader_type::{Scalar, ShaderType};
use std::{fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub struct Project {
    /// Path relative to project root specififying location of vertex shader
    pub vertex_shader_path: String,
    /// Path relative to project root specififying location of fragment shader
    pub fragment_shader_path: String,
}
/// Intermediate represention of shader. Used to convert to production
/// shading langages such as SPIR-V for Vulkan or glsl for webgl2.
pub struct ShaderIR {
    vertex_shader: naga::Module,
    fragment_shader: naga::Module,
}
impl ShaderIR {
    fn parse_path(shader_path: &Path) -> Result<naga::Module> {
        let mut shader_string = String::new();
        File::open(shader_path)?.read_to_string(&mut shader_string)?;
        Ok(wgsl::parse_str(&shader_string)?)
    }
    pub fn compile_from_disk<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path: &Path = path.as_ref();
        let file = File::open(path.join("project.json"))?;
        let project_data: Project = serde_json::from_reader(file)?;
        let vertex_shader = Self::parse_path(&path.join(project_data.vertex_shader_path))?;
        let fragment_shader = Self::parse_path(&path.join(project_data.fragment_shader_path))?;

        Ok(ShaderIR {
            vertex_shader,
            fragment_shader,
        })
    }
}

pub mod vk {
    use super::ShaderType;
    use anyhow::{bail, Result};
    use std::path::Path;
    use thiserror::Error;
    #[derive(Debug, Error)]
    pub enum VulkanConvertError {
        #[error("shader has zero push constants")]
        ZeroPushConstants,
    }
    /// Describes vertex input
    pub struct VertexInput {
        pub binding: u32,
        pub fields: Vec<VertexField>,
    }
    /// Describes a field in a vertex
    pub struct VertexField {
        /// Type in field
        pub ty: ShaderType,
        pub location: u32,
        /// name of field
        pub name: String,
    }
    impl VertexField {
        pub fn size(&self) -> u32 {
            self.ty.size()
        }
    }
    pub struct PushConstant {
        /// type of data in push constant
        pub ty: ShaderType,
    }
    impl PushConstant {
        pub fn size(&self) -> u32 {
            self.ty.size()
        }
    }
    pub struct Texture {
        pub binding: u32,
        pub name: String,
    }
    pub struct Shader {
        /// push constant, assumed to be always in vertex shader
        pub push_constant: PushConstant,
        pub vertex_input: VertexInput,
        /// raw spirv fragment shader data
        pub fragment_spirv_data: Vec<u32>,
        /// raw spirv shader shader data
        pub vertex_spirv_data: Vec<u32>,
        /// textures as global input
        pub textures: Vec<Texture>,
    }
    impl Shader {
        pub fn from_ir(mut shader_ir: super::ShaderIR) -> Result<Self> {
            for var in shader_ir.vertex_shader.global_variables.iter() {
                println!("variable: ");
                println!("{:?}\n\n", var);
            }
            println!("\n\n************ entry points ***********\n\n");
            for point in shader_ir.vertex_shader.entry_points.iter() {
                println!("entry point: ");
                println!("{:?}\n\n", point);
                for arg in point.function.arguments.iter() {
                    println!("{{");
                    if let Some(name) = &arg.name {
                        println!("\tname: {}", name);
                    }
                    println!(
                        "\ttype: {:?}",
                        shader_ir.vertex_shader.types.get_handle(arg.ty).unwrap()
                    );
                    if let Some(binding) = &arg.binding {
                        println!("\tbinding: {:?}", binding);
                    }
                    println!("}}");
                }
                println!("{:?}", point.function.arguments)
            }
            println!("\n\n *********** end entry points *********\n\n");
            let push_constants = shader_ir
                .vertex_shader
                .global_variables
                .iter_mut()
                .map(|(_h, variable)| variable)
                .filter(|variable| variable.class == naga::StorageClass::Uniform)
                .map(|variable| {
                    variable.class = naga::StorageClass::PushConstant;
                    variable
                })
                .collect::<Vec<_>>();
            for push in push_constants.iter() {
                println!("{:?}", push);
                println!("{:?}", shader_ir.vertex_shader.types.get_handle(push.ty));
            }
            if push_constants.len() == 0 {
                bail!("Zero push constants in shader");
            }
            if push_constants.len() > 1 {
                bail!("more then one push constant in shader, there must only ge one push constant per shader")
            }
            let push_type: ShaderType = ShaderType::from_type(
                shader_ir
                    .vertex_shader
                    .types
                    .get_handle(push_constants[0].ty)
                    .unwrap(),
                &shader_ir.vertex_shader.types,
            )?;
            Ok(Self {
                push_constant: PushConstant { ty: push_type },
                vertex_input: todo!("vertex input description"),
                fragment_spirv_data: todo!("spv data"),
                vertex_spirv_data: todo!("spv data"),
                textures: todo!("textures"),
            })
        }
        pub fn write_to_disk<P: AsRef<Path>>(&mut self, path: P) {
            todo!()
        }
    }
}
///
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
