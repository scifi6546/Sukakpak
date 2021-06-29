use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{convert::TryFrom, fs::File, io::Read, path::Path};
pub struct Shader {
    vertex_shader: Module,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AssembledSpirv {
    pub vertex_shader: SpirvModule,
    pub fragment_shader: SpirvModule,
    pub push_constants: Vec<PushConstant>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SpirvModule {
    pub stage: ShaderStage,
    pub binding: u32,
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

impl TryFrom<Shader> for AssembledSpirv {
    type Error = ();
    fn try_from(shader: Shader) -> std::result::Result<Self, Self::Error> {
        todo!()
    }
}
use naga::{front::glsl, Module};
#[derive(Deserialize, Debug)]
pub struct ShaderConfig {
    vertex_shader: ModuleConfig,
}
#[derive(Deserialize, Debug)]
pub struct ModuleConfig {
    path: String,
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
    println!("{:?}", vertex_shader.global_variables);
    println!("\n\n{:?}", vertex_shader.types);
    Ok(Shader { vertex_shader })
}
