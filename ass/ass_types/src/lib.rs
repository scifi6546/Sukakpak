mod shader_type;
pub use shader_type::{Scalar, ShaderType};

use serde::{Deserialize, Serialize};
/// Describes vertex input
#[derive(Deserialize, Serialize, Debug)]
pub struct VertexInput {
    pub binding: u32,
    pub fields: Vec<VertexField>,
}
/// Describes a field in a vertex
#[derive(Deserialize, Serialize, Debug)]
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
