use anyhow::{bail, Result};
///Types that may be used by a shader
#[derive(Clone, Debug, PartialEq, Eq)]
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
            &naga::TypeInner::Scalar { kind, width } => todo!("scalar"),
            &naga::TypeInner::Vector { size, kind, width } => todo!("vector"),
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
            &naga::TypeInner::Atomic { kind, width } => todo!("atomic"),
            &naga::TypeInner::Pointer { base, class } => todo!("pointer"),
            &naga::TypeInner::ValuePointer {
                size,
                kind,
                width,
                class,
            } => todo!("value pointer"),
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
            Self::Struct(s) => s
                .iter()
                .map(|(_name, ty)| ty.size())
                .fold(0, |acc, x| acc + x),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
