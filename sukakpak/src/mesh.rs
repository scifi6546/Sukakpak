use super::{VertexComponent, VertexLayout};
use anyhow::Result;
use nalgebra::{Vector2, Vector3};
use obj::Obj;
use std::path::Path;
use tobj::{load_obj, LoadOptions};
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<u8>,
    pub indices: Vec<u32>,
    pub vertex_layout: VertexLayout,
}
impl Mesh {
    pub fn from_obj(path: &str) -> Result<Self> {
        let (model, mtl) = load_obj(
            path,
            &LoadOptions {
                triangulate: true,

                single_index: true,
                ..Default::default()
            },
        )?;
        let mesh = &model[0].mesh;
        let num_vertices = mesh.positions.len() / 3;
        let vertices = (0..num_vertices)
            .map(|i| {
                (
                    [
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                    ],
                    [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                    [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ],
                )
            })
            .map(|(pos, uv, norm)| {
                [
                    pos[0], pos[1], pos[2], uv[0], uv[1], norm[0], norm[1], norm[2],
                ]
            })
            .flatten()
            .map(|f| f.to_ne_bytes())
            .flatten()
            .collect();
        let indices = mesh.indices.clone();

        Ok(Self {
            indices,
            vertices,
            vertex_layout: VertexLayout {
                components: vec![
                    VertexComponent::Vec3F32,
                    VertexComponent::Vec2F32,
                    VertexComponent::Vec3F32,
                ],
            },
        })
    }
    pub fn new_triangle() -> Self {
        EasyMesh {
            vertices: vec![
                Vertex {
                    position: Vector3::new(-0.5, -0.5, 0.0),
                    uv: Vector2::new(0.0, 0.0),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(0.5, -0.5, 0.0),
                    uv: Vector2::new(1.0, 0.0),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 0.5, 0.0),
                    uv: Vector2::new(0.5, 1.0),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
            ],
            indices: vec![0, 1, 2],
        }
        .into()
    }
    pub fn new_plane() -> Self {
        EasyMesh {
            vertices: vec![
                Vertex {
                    position: Vector3::new(0.0, 0.0, 0.0),
                    uv: Vector2::new(0.0, 0.0),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 0.0, 0.0),
                    uv: Vector2::new(1.0, 0.0),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 1.0, 0.0),
                    uv: Vector2::new(1.0, 1.0),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    uv: Vector2::new(0.0, 1.0),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
            ],
            indices: vec![1, 2, 0, 2, 3, 0],
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
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 0.0, 0.0),
                    uv: Vector2::new(2.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 1.0, 0.0),
                    uv: Vector2::new(1.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 1.0, 1.0),
                    uv: Vector2::new(1.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //face 1
                Vertex {
                    position: Vector3::new(1.0, 0.0, 0.0),
                    uv: Vector2::new(3.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 0.0, 0.0),
                    uv: Vector2::new(3.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(1.0, 1.0, 0.0),
                    uv: Vector2::new(2.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    uv: Vector2::new(2.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //Face 2
                //8
                Vertex {
                    position: Vector3::new(0.0, 0.0, 0.0),
                    uv: Vector2::new(4.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //9
                Vertex {
                    position: Vector3::new(0.0, 0.0, 1.0),
                    uv: Vector2::new(4.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //10
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    uv: Vector2::new(3.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //11
                Vertex {
                    position: Vector3::new(0.0, 1.0, 1.0),
                    uv: Vector2::new(3.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //Face 3
                //12
                Vertex {
                    position: Vector3::new(0.0, 0.0, 1.0),
                    uv: Vector2::new(5.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //13
                Vertex {
                    position: Vector3::new(1.0, 0.0, 1.0),
                    uv: Vector2::new(5.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //14
                Vertex {
                    position: Vector3::new(0.0, 1.0, 1.0),
                    uv: Vector2::new(4.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //15
                Vertex {
                    position: Vector3::new(1.0, 1.0, 1.0),
                    uv: Vector2::new(4.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //face 4
                //16
                Vertex {
                    position: Vector3::new(1.0, 1.0, 1.0),
                    uv: Vector2::new(0.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //17
                Vertex {
                    position: Vector3::new(1.0, 1.0, 0.0),
                    uv: Vector2::new(1.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //18
                Vertex {
                    position: Vector3::new(0.0, 1.0, 1.0),
                    uv: Vector2::new(0.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //19
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    uv: Vector2::new(1.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //face 5
                //20
                Vertex {
                    position: Vector3::new(1.0, 0.0, 1.0),
                    uv: Vector2::new(5.0 / 6.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //21
                Vertex {
                    position: Vector3::new(1.0, 0.0, 0.0),
                    uv: Vector2::new(5.0 / 6.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //22
                Vertex {
                    position: Vector3::new(0.0, 0.0, 1.0),
                    uv: Vector2::new(1.0, 0.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                //23
                Vertex {
                    position: Vector3::new(0.0, 0.0, 0.0),
                    uv: Vector2::new(1.0, 1.0),
                    normal: Vector3::new(-1.0, 0.0, 0.0),
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
    pub normal: Vector3<f32>,
}
impl From<EasyMesh> for Mesh {
    fn from(mesh: EasyMesh) -> Self {
        let len = std::mem::size_of::<Vertex>() * mesh.vertices.len();
        let vertices: Vec<u8> = vec![0; len];
        let mesh_ptr = mesh.vertices.as_ptr();
        let data_ptr = vertices.as_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(mesh_ptr as *const u8, data_ptr as *mut u8, len);
        }
        Mesh {
            vertices,
            indices: mesh.indices,
            vertex_layout: VertexLayout {
                components: vec![
                    VertexComponent::Vec3F32,
                    VertexComponent::Vec2F32,
                    VertexComponent::Vec3F32,
                ],
            },
        }
    }
}
pub struct EasyMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
