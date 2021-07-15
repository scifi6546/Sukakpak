use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fs::File,
    path::Path,
};
#[derive(Serialize, Deserialize, Debug)]
pub struct AssembledSpirv {
    pub vertex_shader: SpirvModule,
    pub fragment_shader: SpirvModule,
    pub textures: HashMap<String, Texture>,
    pub push_constants: Vec<PushConstant>,
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
pub enum ShaderStage {
    Vertex,
    Fragment,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PushConstant {
    pub offset: u32,
    pub size: u32,
    pub stage: ShaderStage,
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
#[derive(Serialize, Deserialize, Debug)]
pub enum ScalarType {
    F32,
    F64,
}
impl Type {
    pub fn size(&self) -> u32 {
        todo!()
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Texture {
    pub binding: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub location: u32,
}
/// Loadable shader, contains description of shader and path of shader
#[derive(Serialize, Deserialize, Debug)]
pub struct LoadableAsmSpv {
    pub vertex_shader: LoadableSpvModule,
    pub fragment_shader: LoadableSpvModule,
    pub textures: HashMap<String, Texture>,
    pub push_constants: Vec<PushConstant>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct LoadableSpvModule {
    pub stage: ShaderStage,
    pub vertex_input_binding: u32,
    pub shader_data_path: Box<Path>,
    pub data_in: Vec<(Type, Location)>,
    pub entry_point: String,
}
impl TryFrom<LoadableAsmSpv> for AssembledSpirv {
    type Error = std::io::Error;
    fn try_from(shader: LoadableAsmSpv) -> std::result::Result<Self, Self::Error> {
        std::result::Result::Ok(Self {
            vertex_shader: shader.vertex_shader.try_into()?,
            fragment_shader: shader.fragment_shader.try_into()?,
            textures: shader.textures,
            push_constants: shader.push_constants,
        })
    }
}
impl TryFrom<LoadableSpvModule> for SpirvModule {
    type Error = std::io::Error;
    fn try_from(shader: LoadableSpvModule) -> std::result::Result<Self, Self::Error> {
        todo!()
    }
}
pub fn load_from_fs<P: AsRef<Path>>(path: P) -> Result<AssembledSpirv> {
    let file = File::open(path.as_ref().join("index.json"))?;
    let shader: LoadableAsmSpv = serde_json::from_reader(file)?;
    Ok(shader.try_into()?)
}
