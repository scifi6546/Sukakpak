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
pub struct Sampler {
    pub binding: u32,
    pub group: u32,
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
    /// samplers used
    pub samplers: Vec<Sampler>,
    /// name of vetex shader entrypoing
    pub vertex_entrypoint: String,
    /// name of fragment shader entrypoing
    pub fragment_entrypoint: String,
}
impl Shader {
    /// Extension to use when writing out shader
    const EXTENSION: &'static str = "ass_spv";
    /// spirv extension
    const SPV_EXTENSION: &'static str = "spv";
    fn get_sampler(shader_ir: &mut super::ShaderIR) -> Result<Vec<Sampler>> {
        Ok(shader_ir
            .module
            .global_variables
            .iter()
            .filter(|(_handle, var)| {
                match shader_ir.module.types.get_handle(var.ty).unwrap().inner {
                    naga::TypeInner::Sampler { .. } => true,
                    _ => false,
                }
            })
            .map(|(_handle, var)| Sampler {
                name: var
                    .name
                    .as_ref()
                    .expect("name does not exist for sampler")
                    .clone(),
                binding: var
                    .binding
                    .as_ref()
                    .expect("binding does not exist for sampler")
                    .binding,
                group: var
                    .binding
                    .as_ref()
                    .expect("group does not exist for sampler")
                    .binding,
            })
            .collect())
    }
    pub fn from_ir(mut shader_ir: super::ShaderIR) -> Result<Self> {
        println!("{:#?}", shader_ir.module);
        let push_constants = shader_ir
            .module
            .global_variables
            .iter_mut()
            .map(|(_h, variable)| variable)
            .filter(|variable| variable.class == naga::StorageClass::Uniform)
            .map(|variable| {
                variable.class = naga::StorageClass::PushConstant;
                variable.binding = None;
                println!("{:?}", variable);
                variable
            })
            .collect::<Vec<_>>();
        if push_constants.len() == 0 {
            bail!("Zero push constants in shader");
        }
        if push_constants.len() > 1 {
            bail!("more then one push constant in shader, there must only ge one push constant per shader")
        }
        let push_type: ShaderType = ShaderType::from_type(
            shader_ir
                .module
                .types
                .get_handle(push_constants[0].ty)
                .unwrap(),
            &shader_ir.module.types,
        )?;
        let vertex_shader_entry_point = shader_ir
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
        let fields = vertex_shader_entry_point[0]
            .function
            .arguments
            .iter()
            .map(|arg| VertexField {
                ty: ShaderType::from_type(
                    shader_ir.module.types.get_handle(arg.ty).unwrap(),
                    &shader_ir.module.types,
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
            &shader_ir.module,
            &shader_ir.info,
            &naga::back::spv::Options::default(),
            Some(&naga::back::spv::PipelineOptions {
                shader_stage: naga::ShaderStage::Vertex,
                entry_point: VERTEX_SHADER_MAIN.to_string(),
            }),
        )?;
        let fragment_spirv_data = naga::back::spv::write_vec(
            &shader_ir.module,
            &shader_ir.info,
            &naga::back::spv::Options::default(),
            Some(&naga::back::spv::PipelineOptions {
                shader_stage: naga::ShaderStage::Fragment,
                entry_point: FRAGMENT_SHADER_MAIN.to_string(),
            }),
        )?;
        let vertex_entrypoint = VERTEX_SHADER_MAIN.to_string();
        let fragment_entrypoint = FRAGMENT_SHADER_MAIN.to_string();
        let textures = shader_ir
            .module
            .global_variables
            .iter()
            .filter(
                |(_h, var)| match shader_ir.module.types.get_handle(var.ty).unwrap().inner {
                    naga::TypeInner::Image { .. } => true,

                    _ => false,
                },
            )
            .map(|(_h, var)| var)
            .map(|tex| Texture {
                binding: tex.binding.as_ref().unwrap().binding,
                name: tex.name.as_ref().unwrap().clone(),
            })
            .collect::<Vec<_>>();
        //doing last validation check

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::PUSH_CONSTANT,
        );

        let info = validator.validate(&shader_ir.module)?;
        let vertex_spirv_data = naga::back::spv::write_vec(
            &shader_ir.module,
            &info,
            &naga::back::spv::Options::default(),
            Some(&naga::back::spv::PipelineOptions {
                shader_stage: naga::ShaderStage::Vertex,
                entry_point: VERTEX_SHADER_MAIN.to_string(),
            }),
        )?;
        let samplers = Self::get_sampler(&mut shader_ir)?;
        let fragment_spirv_data = naga::back::spv::write_vec(
            &shader_ir.module,
            &info,
            &naga::back::spv::Options::default(),
            Some(&naga::back::spv::PipelineOptions {
                shader_stage: naga::ShaderStage::Fragment,
                entry_point: FRAGMENT_SHADER_MAIN.to_string(),
            }),
        )?;
        Ok(Self {
            push_constant: PushConstant { ty: push_type },
            vertex_input: VertexInput { binding: 0, fields },
            fragment_spirv_data,
            vertex_spirv_data,
            samplers,
            vertex_entrypoint,
            fragment_entrypoint,

            textures,
        })
    }
    pub fn to_json_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
    /// Reads from json str, errors if parse
    /// is unsucessfull
    pub fn from_json_str(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
    ///writes to disk with extension ".spv"
    pub fn write_vertex_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref().with_extension(Self::SPV_EXTENSION);
        let mut file = File::create(path)?;
        let data: Vec<u8> = self
            .vertex_spirv_data
            .iter()
            .flat_map(|i| i.to_ne_bytes())
            .collect();
        let num = file.write(&data)?;
        if num != data.len() {
            bail!(
                "failed to write vertex shader to disk, wrote {} bytes needed to write {} bytes",
                num,
                data.len()
            );
        }
        Ok(())
    }
    ///writes to disk with extension ".spv"
    pub fn write_fragment_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref().with_extension(Self::SPV_EXTENSION);
        let mut file = File::create(path)?;
        let data: Vec<u8> = self
            .fragment_spirv_data
            .iter()
            .flat_map(|i| i.to_ne_bytes())
            .collect();
        let num = file.write(&data)?;
        if num != data.len() {
            bail!(
                "failed to write vertex shader to disk, wrote {} bytes needed to write {} bytes",
                num,
                data.len()
            );
        }
        Ok(())
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
