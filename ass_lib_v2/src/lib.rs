#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShaderType {
    Mat4x4(Scalar),
    Vec4(Scalar),
    Vec3(Scalar),
    Vec2(Scalar),
    Scalar(Scalar),
    Struct(Vec<(String, ShaderType)>),
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
    pub struct VertexInputDesc {
        pub binding: u32,
    }
    /// Describes a field in a vertex
    pub struct VertexInput {
        /// Type in field
        pub ty: ShaderType,
        pub location: u32,
        /// name of field
        pub name: String,
    }
    impl VertexInput {
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
        pub vertex_input_desc: VertexInputDesc,
        /// fields in vertex
        pub vertex_fields: Vec<VertexInput>,
        /// raw spirv fragment shader data
        pub fragment_spirv_data: Vec<u32>,
        /// raw spirv shader shader data
        pub vertex_spirv_data: Vec<u32>,
        /// textures as global input
        pub textures: Vec<Texture>,
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
