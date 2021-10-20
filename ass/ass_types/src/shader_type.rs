use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
///Types that may be used by a shader
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ShaderType {
    Mat4x4(Scalar),
    Vec4(Scalar),
    Vec3(Scalar),
    Vec2(Scalar),
    Scalar(Scalar),
    Struct(Vec<(Option<String>, ShaderType)>),
}
impl ShaderType {
    pub fn size(&self) -> u32 {
        match self {
            &Self::Mat4x4(s) => 16 * s.size(),
            &Self::Vec4(s) => 4 * s.size(),
            &Self::Vec3(s) => 3 * s.size(),
            &Self::Vec2(s) => 2 * s.size(),
            &Self::Scalar(s) => s.size(),
            Self::Struct(s) => s.iter().map(|(_name, ty)| ty.size()).sum(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Scalar {
    F32,
    U32,
}
impl Scalar {
    pub fn size(&self) -> u32 {
        match *self {
            Self::F32 => std::mem::size_of::<f32>() as u32,
            Self::U32 => std::mem::size_of::<u32>() as u32,
        }
    }
}
