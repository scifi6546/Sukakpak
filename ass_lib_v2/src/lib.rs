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
    pub vertex_shader_path: String,
    /// Path relative to project root specififying location of fragment shader
    pub fragment_shader_path: String,
}
/// Intermediate represention of shader. Used to convert to production
/// shading langages such as SPIR-V for Vulkan or glsl for webgl2.
pub struct ShaderIR {
    vertex_shader: IrModule,
    fragment_shader: IrModule,
}
pub struct IrModule {
    module: naga::Module,
    info: naga::valid::ModuleInfo,
}
impl ShaderIR {
    fn parse_path(shader_path: &Path) -> Result<IrModule> {
        let mut shader_string = String::new();
        File::open(shader_path)?.read_to_string(&mut shader_string)?;
        let module = wgsl::parse_str(&shader_string)?;
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::PUSH_CONSTANT,
        );

        let info = validator.validate(&module)?;
        Ok(IrModule { module, info })
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
    use super::{ShaderType, FRAGMENT_SHADER_MAIN, VERTEX_SHADER_MAIN};
    use anyhow::{bail, Result};
    use serde::{Deserialize, Serialize};
    use std::{
        fs::File,
        io::{Read, Write},
        path::Path,
    };
    use thiserror::Error;
    #[derive(Debug, Error)]
    pub enum VulkanConvertError {
        #[error("shader has zero push constants")]
        ZeroPushConstants,
    }
    /// Describes vertex input
    #[derive(Deserialize, Serialize, Debug)]
    pub struct VertexInput {
        pub binding: u32,
        pub fields: Vec<VertexField>,
    }
    /// Describes a field in a vertex
    #[derive(Deserialize, Serialize, Debug)]
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
    #[derive(Deserialize, Serialize, Debug)]
    pub struct PushConstant {
        /// type of data in push constant
        pub ty: ShaderType,
    }
    impl PushConstant {
        pub fn size(&self) -> u32 {
            self.ty.size()
        }
    }
    #[derive(Deserialize, Serialize, Debug)]
    pub struct Texture {
        pub binding: u32,
        pub name: String,
    }
    #[derive(Deserialize, Serialize, Debug)]
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
        /// Extension to use when writing out shader
        const EXTENSION: &'static str = "ass_spv";
        pub fn from_ir(mut shader_ir: super::ShaderIR) -> Result<Self> {
            for (_handle, var) in shader_ir.vertex_shader.module.global_variables.iter() {
                println!("variable: ");
                println!("{:?}\n\n", var);
                println!(
                    "\n\ntype: {:?}\n\n",
                    shader_ir
                        .vertex_shader
                        .module
                        .types
                        .get_handle(var.ty)
                        .unwrap()
                );
            }
            let push_constants = shader_ir
                .vertex_shader
                .module
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
                println!(
                    "{:?}",
                    shader_ir.vertex_shader.module.types.get_handle(push.ty)
                );
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
                    .module
                    .types
                    .get_handle(push_constants[0].ty)
                    .unwrap(),
                &shader_ir.vertex_shader.module.types,
            )?;
            let vertex_shader_entry_point = shader_ir
                .vertex_shader
                .module
                .entry_points
                .iter()
                .filter(|entry| entry.stage == naga::ShaderStage::Vertex)
                .collect::<Vec<_>>();

            if vertex_shader_entry_point.len() == 0 {
                bail!("there must be a vertex entry point in shader");
            }
            if vertex_shader_entry_point.len() > 1 {
                bail!(
                    "there must be only one vertex entry point in shader, got {} entry points",
                    vertex_shader_entry_point.len()
                );
            }
            println!("\n\n************ entry points ***********\n\n");
            for point in vertex_shader_entry_point.iter() {
                println!("entry point: ");
                println!("{:?}\n\n", point);
                for arg in point.function.arguments.iter() {
                    println!("{{");
                    if let Some(name) = &arg.name {
                        println!("\tname: {}", name);
                    }
                    println!(
                        "\ttype: {:?}",
                        shader_ir
                            .vertex_shader
                            .module
                            .types
                            .get_handle(arg.ty)
                            .unwrap()
                    );
                    if let Some(binding) = &arg.binding {
                        println!("\tbinding: {:?}", binding);
                    }
                    println!("}}");
                }
                println!("{:?}", point.function.arguments)
            }
            println!("\n\n *********** end entry points *********\n\n");
            let fields = vertex_shader_entry_point[0]
                .function
                .arguments
                .iter()
                .map(|arg| VertexField {
                    ty: ShaderType::from_type(
                        shader_ir
                            .vertex_shader
                            .module
                            .types
                            .get_handle(arg.ty)
                            .unwrap(),
                        &shader_ir.vertex_shader.module.types,
                    )
                    .expect("failed to convert type"),
                    location: match arg.binding.as_ref().unwrap() {
                        naga::Binding::BuiltIn(_) => panic!("invalid vertex input"),
                        naga::Binding::Location {
                            location,
                            interpolation,
                            sampling,
                        } => *location,
                    },
                    name: arg.name.as_ref().unwrap().clone(),
                })
                .collect();
            let vertex_spirv_data = naga::back::spv::write_vec(
                &shader_ir.vertex_shader.module,
                &shader_ir.vertex_shader.info,
                &naga::back::spv::Options::default(),
                Some(&naga::back::spv::PipelineOptions {
                    shader_stage: naga::ShaderStage::Vertex,
                    entry_point: VERTEX_SHADER_MAIN.to_string(),
                }),
            )?;
            let fragment_spirv_data = naga::back::spv::write_vec(
                &shader_ir.vertex_shader.module,
                &shader_ir.vertex_shader.info,
                &naga::back::spv::Options::default(),
                Some(&naga::back::spv::PipelineOptions {
                    shader_stage: naga::ShaderStage::Fragment,
                    entry_point: FRAGMENT_SHADER_MAIN.to_string(),
                }),
            )?;
            let textures = shader_ir
                .vertex_shader
                .module
                .global_variables
                .iter()
                .filter(|(_h, var)| {
                    match shader_ir
                        .vertex_shader
                        .module
                        .types
                        .get_handle(var.ty)
                        .unwrap()
                        .inner
                    {
                        naga::TypeInner::Image { .. } => true,

                        _ => false,
                    }
                })
                .map(|(_h, var)| var)
                .map(|tex| Texture {
                    binding: tex.binding.as_ref().unwrap().binding,
                    name: tex.name.as_ref().unwrap().clone(),
                })
                .collect::<Vec<_>>();
            println!("\n\n*****************  textures *******************\n\n");
            for t in textures.iter() {
                println!("tex: {:?}", t);
            }
            println!("\n\n*****************  end textures *******************\n\n");

            Ok(Self {
                push_constant: PushConstant { ty: push_type },
                vertex_input: VertexInput { binding: 0, fields },
                fragment_spirv_data,
                vertex_spirv_data,
                textures,
            })
        }
        pub fn to_json_string(&self) -> Result<String> {
            Ok(serde_json::to_string(self)?)
        }
        /// writes to disk with extension ".ass_spv"
        pub fn write_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<()> {
            let json_string = self.to_json_string()?;
            let new_path = path.as_ref().with_extension(Self::EXTENSION);
            let mut file = File::create(new_path)?;
            file.write_all(json_string.as_bytes())?;
            Ok(())
        }
        /// reads from disk, setting extension to ".ass_spv"
        pub fn read_from_disk<P: AsRef<Path>>(path: P) -> Result<Self> {
            let new_path = path.as_ref().with_extension(Self::EXTENSION);
            let file = File::open(new_path)?;
            let out: Self = serde_json::from_reader(file)?;
            Ok(out)
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
