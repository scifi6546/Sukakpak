use anyhow::{bail, Result};
use ass_types::{Scalar, ShaderType};
/// gets type from naga type
pub fn type_from_naga(
    ty: &naga::Type,
    arena: &naga::UniqueArena<naga::Type>,
) -> Result<ShaderType> {
    match &ty.inner {
        &naga::TypeInner::Scalar { kind, width } => {
            Ok(ShaderType::Scalar(scalar_from_naga(&kind, width)?))
        }
        &naga::TypeInner::Vector { size, kind, width } => match size {
            naga::VectorSize::Bi => match kind {
                naga::ScalarKind::Sint => bail!("signed int not supported"),
                naga::ScalarKind::Uint => match width {
                    4 => Ok(ShaderType::Vec2(Scalar::U32)),
                    _ => bail!("invalid scalar width"),
                },
                naga::ScalarKind::Float => match width {
                    4 => Ok(ShaderType::Vec2(Scalar::F32)),
                    _ => bail!("invalid scalar width"),
                },
                naga::ScalarKind::Bool => bail!("bool not (yet) supported as kind for vec"),
            },
            naga::VectorSize::Tri => match kind {
                naga::ScalarKind::Sint => bail!("signed int not supported"),
                naga::ScalarKind::Uint => match width {
                    4 => Ok(ShaderType::Vec3(Scalar::U32)),
                    _ => bail!("invalid scalar width"),
                },
                naga::ScalarKind::Float => match width {
                    4 => Ok(ShaderType::Vec3(Scalar::F32)),
                    _ => bail!("invalid scalar width"),
                },
                naga::ScalarKind::Bool => bail!("bool not (yet) supported as kind for vec"),
            },
            naga::VectorSize::Quad => match kind {
                naga::ScalarKind::Sint => bail!("signed int not supported"),
                naga::ScalarKind::Uint => match width {
                    4 => Ok(ShaderType::Vec4(Scalar::U32)),
                    _ => bail!("invalid scalar width"),
                },
                naga::ScalarKind::Float => match width {
                    4 => Ok(ShaderType::Vec4(Scalar::F32)),
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
                    4 => Ok(ShaderType::Mat4x4(Scalar::F32)),
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
        } => Ok(ShaderType::Struct(
            members
                .iter()
                .map(|member| {
                    (
                        member.name.clone(),
                        type_from_naga(arena.get_handle(member.ty).unwrap(), arena)
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
pub fn scalar_from_naga(kind: &naga::ScalarKind, width: u8) -> Result<Scalar> {
    match kind {
        &naga::ScalarKind::Uint => match width {
            4 => Ok(Scalar::U32),
            8 => bail!("64 bit unsigned ints not yet supported"),
            _ => bail!("unsupported int width"),
        },
        &naga::ScalarKind::Float => match width {
            4 => Ok(Scalar::F32),
            8 => bail!("64 bit unsigned floats not yet supported"),
            _ => bail!("unsupported int width"),
        },
        &naga::ScalarKind::Sint => bail!("signed ints not supported yet"),
        &naga::ScalarKind::Bool => bail!("bools are not supported yet"),
    }
}
