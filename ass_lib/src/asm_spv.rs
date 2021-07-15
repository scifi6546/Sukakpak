use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
