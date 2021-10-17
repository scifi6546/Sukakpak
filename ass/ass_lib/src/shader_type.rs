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
    pub fn from_type(ty: &naga::Type, arena: &naga::UniqueArena<naga::Type>) -> Result<Self> {
        match &ty.inner {
            &naga::TypeInner::Scalar { kind, width } => {
                Ok(Self::Scalar(Scalar::from_naga(&kind, width)?))
            }
            &naga::TypeInner::Vector { size, kind, width } => match size {
                naga::VectorSize::Bi => match kind {
                    naga::ScalarKind::Sint => bail!("signed int not supported"),
                    naga::ScalarKind::Uint => match width {
                        4 => Ok(Self::Vec2(Scalar::U32)),
                        _ => bail!("invalid scalar width"),
                    },
                    naga::ScalarKind::Float => match width {
                        4 => Ok(Self::Vec2(Scalar::F32)),
                        _ => bail!("invalid scalar width"),
                    },
                    naga::ScalarKind::Bool => bail!("bool not (yet) supported as kind for vec"),
                },
                naga::VectorSize::Tri => match kind {
                    naga::ScalarKind::Sint => bail!("signed int not supported"),
                    naga::ScalarKind::Uint => match width {
                        4 => Ok(Self::Vec3(Scalar::U32)),
                        _ => bail!("invalid scalar width"),
                    },
                    naga::ScalarKind::Float => match width {
                        4 => Ok(Self::Vec3(Scalar::F32)),
                        _ => bail!("invalid scalar width"),
                    },
                    naga::ScalarKind::Bool => bail!("bool not (yet) supported as kind for vec"),
                },
                naga::VectorSize::Quad => match kind {
                    naga::ScalarKind::Sint => bail!("signed int not supported"),
                    naga::ScalarKind::Uint => match width {
                        4 => Ok(Self::Vec4(Scalar::U32)),
                        _ => bail!("invalid scalar width"),
                    },
                    naga::ScalarKind::Float => match width {
                        4 => Ok(Self::Vec4(Scalar::F32)),
                        _ => bail!("invalid scalar width"),
                    },
                    naga::ScalarKind::Bool => bail!("bool not (yet) supported as kind for vec"),
                },
            },
            &naga::TypeInner::Matrix {
                columns,
                rows,
                width,
            } => match rows {
                naga::VectorSize::Bi => todo!("matrix with two rows"),
                naga::VectorSize::Tri => todo!("matrix with three rows"),
                naga::VectorSize::Quad => match columns {
                    naga::VectorSize::Bi => todo!("4x2 matrix"),
                    naga::VectorSize::Tri => todo!("4x3 matrix"),
                    naga::VectorSize::Quad => match width {
                        4 => Ok(Self::Mat4x4(Scalar::F32)),
                        _ => bail!("matricies with non f32 types are not currently supported"),
                    },
                },
            },
            &naga::TypeInner::Atomic { .. } => todo!("atomic"),
            &naga::TypeInner::Pointer { .. } => todo!("pointer"),
            &naga::TypeInner::ValuePointer { .. } => todo!("value pointer"),
            &naga::TypeInner::Array { base, size, stride } => todo!("array"),
            naga::TypeInner::Struct {
                top_level,
                members,
                span,
            } => Ok(Self::Struct(
                members
                    .iter()
                    .map(|member| {
                        (
                            member.name.clone(),
                            Self::from_type(arena.get_handle(member.ty).unwrap(), arena)
                                .expect("failed to build struct"),
                        )
                    })
                    .collect(),
            )),
            &naga::TypeInner::Image {
                dim,
                arrayed,
                class,
            } => todo!("image"),
            &naga::TypeInner::Sampler { comparison } => todo!("sampler"),
        }
    }
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
    fn from_naga(kind: &naga::ScalarKind, width: u8) -> Result<Self> {
        match kind {
            &naga::ScalarKind::Uint => match width {
                4 => Ok(Self::U32),
                8 => bail!("64 bit unsigned ints not yet supported"),
                _ => bail!("unsupported int width"),
            },
            &naga::ScalarKind::Float => match width {
                4 => Ok(Self::F32),
                8 => bail!("64 bit unsigned floats not yet supported"),
                _ => bail!("unsupported int width"),
            },
            &naga::ScalarKind::Sint => bail!("signed ints not supported yet"),
            &naga::ScalarKind::Bool => bail!("bools are not supported yet"),
        }
    }
}
