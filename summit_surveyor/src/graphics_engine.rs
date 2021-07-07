#[cfg(not(target_arch = "wasm32"))]
mod gfx;
mod mesh;
#[cfg(target_arch = "wasm32")]
mod webgl;
#[cfg(not(target_arch = "wasm32"))]
pub use gfx::*;
pub use mesh::{ItemDesc, Mesh, Vertex};
use nalgebra::{Matrix4, Vector3};
#[cfg(target_arch = "wasm32")]
pub use webgl::*;
