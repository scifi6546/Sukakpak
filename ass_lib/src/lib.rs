use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, convert::TryFrom, fs::File, io::Read, path::Path};
mod assembled_spv;
pub use assembled_spv::AssembledSpirv;

pub struct Shader {
    vertex_shader: Module,
    vertex_info: ShaderDescription,
    fragment_shader: Module,
    fragment_info: ShaderDescription,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Texture {
    pub binding: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SpirvModule {
    pub stage: ShaderStage,
    pub vertex_input_binding: u32,
    pub data: Vec<u32>,
    pub data_in: Vec<(Type, Location)>,
    pub entry_point: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    Scalar(ScalarType),
    Vec2(ScalarType),
    Vec3(ScalarType),
    Vec4(ScalarType),
    Mat2(ScalarType),
    Mat3(ScalarType),
    Mat4(ScalarType),
}
impl Type {
    pub fn size(&self) -> u32 {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ScalarType {
    F32,
    F64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub location: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PushConstant {
    pub offset: u32,
    pub size: u32,
    pub stage: ShaderStage,
}
//info for shader description that accompanies shader module
#[derive(Serialize, Deserialize, Debug)]
pub struct ShaderDescription {
    stage: ShaderStage,
    vertex_input_binding: u32,
}
impl TryFrom<Shader> for AssembledSpirv {
    type Error = anyhow::Error;
    fn try_from(shader: Shader) -> std::result::Result<Self, Self::Error> {
        Ok(AssembledSpirv {
            vertex_shader: Shader::get_module(shader.vertex_shader, shader.vertex_info)?,
            fragment_shader: Shader::get_module(shader.fragment_shader, shader.fragment_info)?,
            push_constants: todo!(),
            textures: todo!(),
        })
    }
}
impl Shader {
    fn get_module(module: naga::Module, info: ShaderDescription) -> Result<SpirvModule> {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::PUSH_CONSTANT,
        );
        let module_info = validator.validate(&module)?;
        let data = naga::back::spv::write_vec(
            &module,
            &module_info,
            &naga::back::spv::Options {
                lang_version: (1, 5),
                flags: naga::back::spv::WriterFlags::empty(),
                capabilities: None,
            },
        )?;
        Ok(SpirvModule {
            stage: info.stage,
            vertex_input_binding: info.vertex_input_binding,
            data,
            data_in: module
                .entry_points
                .iter()
                .next()
                .expect("no entry points in shader")
                .function
                .arguments
                .iter()
                .filter(|arg| arg.binding.is_some())
                .map(|arg| {
                    (
                        (&module.types[arg.ty]).into(),
                        Location {
                            location: match arg.binding.clone().expect("location found") {
                                naga::Binding::Location { location, .. } => location,
                                _ => todo!(),
                            },
                        },
                    )
                })
                .collect(),
            entry_point: module
                .entry_points
                .iter()
                .next()
                .expect("no entry points in shader")
                .name
                .clone(),
        })
    }
}
fn to_scalar(kind: naga::ScalarKind, size: naga::Bytes) -> ScalarType {
    match kind {
        naga::ScalarKind::Sint => todo!("not yet supported"),
        naga::ScalarKind::Uint => todo!("not yet supported"),
        naga::ScalarKind::Float => match size {
            4 => ScalarType::F32,
            8 => ScalarType::F64,
            _ => panic!("invalid float size"),
        },
        naga::ScalarKind::Bool => todo!("not yet supported"),
    }
}
impl From<&naga::Type> for Type {
    fn from(ty: &naga::Type) -> Self {
        match ty.inner {
            naga::TypeInner::Scalar { kind, width } => Type::Scalar(to_scalar(kind, width)),
            naga::TypeInner::Vector { size, kind, width } => match size {
                naga::VectorSize::Bi => Type::Vec2(to_scalar(kind, width)),
                naga::VectorSize::Tri => Type::Vec3(to_scalar(kind, width)),
                naga::VectorSize::Quad => Type::Vec4(to_scalar(kind, width)),
            },
            naga::TypeInner::Matrix {
                columns,
                rows,
                width,
            } => match columns {
                naga::VectorSize::Bi => match rows {
                    naga::VectorSize::Bi => match width {
                        4 => Type::Mat2(ScalarType::F32),
                        8 => Type::Mat2(ScalarType::F64),
                        _ => panic!("unsupported floating point size"),
                    },
                    _ => panic!("only square matricies are supported"),
                },
                naga::VectorSize::Tri => match rows {
                    naga::VectorSize::Tri => match width {
                        4 => Type::Mat3(ScalarType::F32),
                        8 => Type::Mat3(ScalarType::F64),
                        _ => panic!("unsupported floating point size"),
                    },
                    _ => panic!("only square matricies are supported"),
                },
                naga::VectorSize::Quad => match rows {
                    naga::VectorSize::Quad => match width {
                        4 => Type::Mat4(ScalarType::F32),
                        8 => Type::Mat4(ScalarType::F64),
                        _ => panic!("unsupported floating point size"),
                    },
                    _ => panic!("only square matricies are supported"),
                },
            },
            _ => todo!("type is currently unsupported"),
        }
    }
}
use naga::{front::glsl, Module};
#[derive(Deserialize, Debug)]
pub struct ShaderConfig {
    vertex_shader: ModuleConfig,
    fragment_shader: ModuleConfig,
}
#[derive(Deserialize, Debug)]
pub struct ModuleConfig {
    path: String,
    info: ShaderDescription,
    entry_point: EntryPoint,
}
#[derive(Deserialize, Debug)]
pub struct EntryPoint {
    name: String,
    stage: ShaderStage,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

pub fn load_directory(path: &Path) -> Result<Shader> {
    let config: ShaderConfig = {
        let config_file = File::open(path.join("config.json"))?;
        serde_json::from_reader(config_file)?
    };
    let mut file = File::open(path.join(config.vertex_shader.path))?;
    let mut frag_str = String::new();
    file.read_to_string(&mut frag_str)?;
    let entry_points = [config.vertex_shader.entry_point]
        .iter()
        .map(|e| {
            (
                e.name.clone(),
                match e.stage {
                    ShaderStage::Fragment => naga::ShaderStage::Fragment,
                    ShaderStage::Vertex => naga::ShaderStage::Vertex,
                },
            )
        })
        .collect();
    let vertex_shader = glsl::parse_str(
        &frag_str,
        &glsl::Options {
            entry_points,
            defines: naga::FastHashMap::default(),
        },
    )?;
    let fragment_shader = {
        let mut file = File::open(path.join(config.fragment_shader.path))?;
        let mut frag_str = String::new();
        file.read_to_string(&mut frag_str)?;
        let entry_points = [config.fragment_shader.entry_point]
            .iter()
            .map(|e| {
                (
                    e.name.clone(),
                    match e.stage {
                        ShaderStage::Fragment => naga::ShaderStage::Fragment,
                        ShaderStage::Vertex => naga::ShaderStage::Vertex,
                    },
                )
            })
            .collect();
        println!("{}", frag_str);
        let s = glsl::parse_str(
            &frag_str,
            &glsl::Options {
                entry_points,
                defines: naga::FastHashMap::default(),
            },
        );
        println!("{:#?}", s);
        s?
    };
    println!("{:#?}", vertex_shader);
    Ok(Shader {
        vertex_shader,
        vertex_info: config.vertex_shader.info,
        fragment_shader,
        fragment_info: config.fragment_shader.info,
    })
}
