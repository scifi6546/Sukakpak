pub use anyhow;
use anyhow::Result;
use naga::front::wgsl;
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub struct Project {
    /// Path relative to project root specififying location of vertex shader
    pub vertex_shader_path: String,
    /// Path relative to project root specififying location of fragment shader
    pub fragment_shader_path: String,
}
/// Intermediate represention of shader. Used to convert to production
/// shading langages such as SPIR-V for Vulkan or glsl for webgl2.
pub struct ShaderIR {
    vertex_shader: naga::Module,
    fragment_shader: naga::Module,
}
impl ShaderIR {
    fn parse_path(shader_path: &Path) -> Result<naga::Module> {
        let mut shader_string = String::new();
        File::open(shader_path)?.read_to_string(&mut shader_string)?;
        Ok(wgsl::parse_str(&shader_string)?)
    }
    pub fn compile_from_disk<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path: &Path = path.as_ref();
        let file = File::open(path.join("project.json"))?;
        let project_data: Project = serde_json::from_reader(file)?;
        let vertex_shader = Self::parse_path(&path.join(project_data.vertex_shader_path))?;
        let fragment_shader = Self::parse_path(&path.join(project_data.fragment_shader_path))?;

        Ok(ShaderIR {
            vertex_shader,
            fragment_shader,
        })
    }
}
///Types that may be used by a shader
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShaderType {
    Mat4x4(Scalar),
    Vec4(Scalar),
    Vec3(Scalar),
    Vec2(Scalar),
    Scalar(Scalar),
    Struct(Vec<(String, ShaderType)>),
}
impl From<&naga::Type> for ShaderType {
    fn from(ty: &naga::Type) -> Self {
        match &ty.inner {
            &naga::TypeInner::Scalar { kind, width } => todo!("scalar"),
            &naga::TypeInner::Vector { size, kind, width } => todo!("vector"),
            &naga::TypeInner::Matrix {
                columns,
                rows,
                width,
            } => todo!("matrix"),
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
            } => todo!("struct"),
            &naga::TypeInner::Image {
                dim,
                arrayed,
                class,
            } => todo!("image"),
            &naga::TypeInner::Sampler { comparison } => todo!("sampler"),
        }
    }
}
impl ShaderType {
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

pub mod vk {
    use super::ShaderType;
    use anyhow::{bail, Result};
    use std::path::Path;
    use thiserror::Error;
    #[derive(Debug, Error)]
    pub enum VulkanConvertError {
        #[error("shader has zero push constants")]
        ZeroPushConstants,
    }
    /// Describes vertex input
    pub struct VertexInput {
        pub binding: u32,
        pub fields: Vec<VertexField>,
    }
    /// Describes a field in a vertex
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
    pub struct PushConstant {
        /// type of data in push constant
        pub ty: ShaderType,
    }
    impl PushConstant {
        pub fn size(&self) -> u32 {
            self.ty.size()
        }
    }
    pub struct Texture {
        pub binding: u32,
        pub name: String,
    }
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
    }
    impl Shader {
        pub fn from_ir(shader_ir: super::ShaderIR) -> Result<Self> {
            let push_constants = shader_ir
                .vertex_shader
                .global_variables
                .iter()
                .map(|(_h, variable)| variable)
                .filter(|variable| variable.class == naga::StorageClass::Uniform)
                .collect::<Vec<_>>();
            for push in push_constants.iter() {
                println!("{:?}", push);
                println!("{:?}", shader_ir.vertex_shader.types.get_handle(push.ty));
            }
            if push_constants.len() == 0 {
                bail!("Zero push constants in shader");
            }
            let push_type: ShaderType = shader_ir
                .vertex_shader
                .types
                .get_handle(push_constants[0].ty)
                .unwrap()
                .into();
            Ok(Self {
                push_constant: PushConstant { ty: push_type },
                vertex_input: todo!("vertex input description"),
                fragment_spirv_data: todo!("spv data"),
                vertex_spirv_data: todo!("spv data"),
                textures: todo!("textures"),
            })
        }
        pub fn write_to_disk<P: AsRef<Path>>(&mut self, path: P) {
            todo!()
        }
    }
}
///
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
