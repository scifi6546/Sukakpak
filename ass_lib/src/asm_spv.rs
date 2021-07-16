use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::prelude::*, path::Path};
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
impl ScalarType {
    pub fn size(&self) -> u32 {
        match self {
            &Self::F32 => std::mem::size_of::<f32>() as u32,
            &Self::F64 => std::mem::size_of::<f64>() as u32,
        }
    }
}
impl Type {
    pub fn size(&self) -> u32 {
        match &self {
            &Self::Scalar(ty) => 1 * ty.size(),
            &Self::Vec2(ty) => 2 * ty.size(),
            &Self::Vec3(ty) => 3 * ty.size(),
            &Self::Vec4(ty) => 4 * ty.size(),
            &Self::Mat2(ty) => 2 * 2 * ty.size(),
            &Self::Mat3(ty) => 3 * 3 * ty.size(),
            &Self::Mat4(ty) => 4 * 4 * ty.size(),
        }
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
impl LoadableAsmSpv {
    fn to_spirv(self, path: &Path) -> Result<AssembledSpirv> {
        Ok(AssembledSpirv {
            vertex_shader: self.vertex_shader.to_spirv(path)?,
            fragment_shader: self.fragment_shader.to_spirv(path)?,
            textures: self.textures,
            push_constants: self.push_constants,
        })
    }
}
impl LoadableSpvModule {
    fn to_spirv(self, path: &Path) -> Result<SpirvModule> {
        let path = path.join(self.shader_data_path);
        let mut file = File::open(path)?;
        let mut data_u8 = vec![];
        file.read_to_end(&mut data_u8)?;
        assert!(data_u8.len() % 4 == 0);
        let mut data = vec![];
        data.reserve(data_u8.len() / 4);
        let data_len = data_u8.len() / 4;
        for i in 0..data_len {
            data.push(u32::from_ne_bytes([
                data_u8[i * 4],
                data_u8[i * 4 + 1],
                data_u8[i * 4 + 2],
                data_u8[i * 4 + 3],
            ]));
        }

        Ok(SpirvModule {
            stage: self.stage,
            vertex_input_binding: self.vertex_input_binding,
            data,
            data_in: self.data_in,
            entry_point: self.entry_point,
        })
    }
}
pub fn load_from_fs<P: AsRef<Path>>(path: P) -> Result<AssembledSpirv> {
    let file = File::open(path.as_ref().join("index.json"))?;
    let shader: LoadableAsmSpv = serde_json::from_reader(file)?;
    Ok(shader.to_spirv(path.as_ref())?)
}
