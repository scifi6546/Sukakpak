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
    pub fn new_cube() -> Self {
        EasyMesh {
            vertices: vec![
                //face 0
                Vertex {
                    position: Vector3::new(1.0, 0.0, 1.0),
                    uv: Vector2::new(2.0 / 6.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 0.0, 0.0),
                    uv: Vector2::new(2.0 / 6.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 1.0, 0.0),
                    uv: Vector2::new(1.0 / 6.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 1.0, 1.0),
                    uv: Vector2::new(1.0 / 6.0, 0.0),
                },
                //face 1
                Vertex {
                    position: Vector3::new(1.0, 0.0, 0.0),
                    uv: Vector2::new(3.0 / 6.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 0.0, 0.0),
                    uv: Vector2::new(3.0 / 6.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 1.0, 0.0),
                    uv: Vector2::new(2.0 / 6.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    uv: Vector2::new(2.0 / 6.0, 1.0),
                },
                //Face 2
                //8
                Vertex {
                    position: Vector3::new(0.0, 0.0, 0.0),
                    uv: Vector2::new(4.0 / 6.0, 0.0),
                },
                //9
                Vertex {
                    position: Vector3::new(0.0, 0.0, 1.0),
                    uv: Vector2::new(4.0 / 6.0, 1.0),
                },
                //10
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    uv: Vector2::new(3.0 / 6.0, 0.0),
                },
                //11
                Vertex {
                    position: Vector3::new(0.0, 1.0, 1.0),
                    uv: Vector2::new(3.0 / 6.0, 1.0),
                },
                //Face 3
                //12
                Vertex {
                    position: Vector3::new(0.0, 0.0, 1.0),
                    uv: Vector2::new(5.0 / 6.0, 0.0),
                },
                //13
                Vertex {
                    position: Vector3::new(1.0, 0.0, 1.0),
                    uv: Vector2::new(5.0 / 6.0, 1.0),
                },
                //14
                Vertex {
                    position: Vector3::new(0.0, 1.0, 1.0),
                    uv: Vector2::new(4.0 / 6.0, 0.0),
                },
                //15
                Vertex {
                    position: Vector3::new(1.0, 1.0, 1.0),
                    uv: Vector2::new(4.0 / 6.0, 1.0),
                },
                //face 4
                //16
                Vertex {
                    position: Vector3::new(1.0, 1.0, 1.0),
                    uv: Vector2::new(0.0, 0.0),
                },
                //17
                Vertex {
                    position: Vector3::new(1.0, 1.0, 0.0),
                    uv: Vector2::new(1.0 / 6.0, 0.0),
                },
                //18
                Vertex {
                    position: Vector3::new(0.0, 1.0, 1.0),
                    uv: Vector2::new(0.0, 1.0),
                },
                //19
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    uv: Vector2::new(1.0 / 6.0, 1.0),
                },
                //face 5
                //20
                Vertex {
                    position: Vector3::new(1.0, 0.0, 1.0),
                    uv: Vector2::new(5.0 / 6.0, 0.0),
                },
                //21
                Vertex {
                    position: Vector3::new(1.0, 0.0, 0.0),
                    uv: Vector2::new(5.0 / 6.0, 1.0),
                },
                //22
                Vertex {
                    position: Vector3::new(0.0, 0.0, 1.0),
                    uv: Vector2::new(1.0, 0.0),
                },
                //23
                Vertex {
                    position: Vector3::new(0.0, 0.0, 0.0),
                    uv: Vector2::new(1.0, 1.0),
                },
            ],
            indices: vec![
                [0, 1, 2],
                [0, 2, 3],
                [5, 6, 4],
                [5, 7, 6],
                [8, 9, 10],
                [10, 9, 11],
                [12, 13, 15],
                [12, 15, 14],
                [16, 17, 19],
                [16, 19, 18],
                [20, 23, 21],
                [20, 22, 23],
            ]
            .iter()
            .flatten()
            .copied()
            .collect(),
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
