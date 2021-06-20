use super::VertexLayout;
use nalgebra::{Vector2, Vector3};
pub struct Mesh {
    pub verticies: Vec<u8>,
    pub indices: Vec<u32>,
    pub vertex_layout: VertexLayout,
}
impl Mesh {
    pub fn new_triangle() -> Self {
        EasyMesh {
            vertices: vec![
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
    pub position: Vector3<f32>,
    pub uv: Vector2<f32>,
}
impl From<EasyMesh> for Mesh {
    fn from(mesh: EasyMesh) -> Self {
        let len = std::mem::size_of::<Vertex>() * mesh.vertices.len();
        let verticies: Vec<u8> = vec![0; len];
        let mesh_ptr = mesh.vertices.as_ptr();
        let data_ptr = verticies.as_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(mesh_ptr as *const u8, data_ptr as *mut u8, len);
        }
        Mesh {
            verticies,
            indices: mesh.indices,
            vertex_layout: VertexLayout::XYZ_UV_F32,
        }
    }
}
pub struct EasyMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
