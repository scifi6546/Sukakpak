use nalgebra::{Vector2, Vector3};
pub struct Mesh {
    data: Vec<u8>,
    indices: Vec<u32>,
}
impl Mesh {
    pub fn new_triangle() -> Self {
        EasyMesh {
            verticies: vec![
                Vertex {
                    position: Vector3::new(-0.5, -0.5, 0.0),
                    uv: Vector2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.5, -0.5, 0.0),
                    uv: Vector2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 0.5, 0.0),
                    uv: Vector2::new(0.5, 1.0),
                },
            ],
            indices: vec![0, 1, 2],
        }
        .into()
    }
}
pub struct Vertex {
    position: Vector3<f32>,
    uv: Vector2<f32>,
}
impl From<EasyMesh> for Mesh {
    fn from(data: EasyMesh) -> Self {
        todo!()
    }
}
pub struct EasyMesh {
    pub verticies: Vec<Vertex>,
    pub indices: Vec<u32>,
}
