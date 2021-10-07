use std::time::Duration;
use sukakpak::{
    image,
    nalgebra::{Matrix4, Vector2},
    Context, ContextTrait, CreateInfo, DrawableTexture, Event, Mesh, MeshAsset, Renderable,
};
pub struct TestGame {
    cube: Mesh,
}
impl Renderable for TestGame {
    fn init(mut context: Context) -> Self {
        let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 20, 0, 0]));
        let texture = context
            .build_texture(&image)
            .expect("failed to build texture");
        let cube = context
            .build_mesh(MeshAsset::new_cube(), DrawableTexture::Texture(&texture))
            .expect("failed to build mesh");
        // todo: figure out shaders
        Self { cube }
    }
    fn render_frame(&mut self, events: &[Event], mut context: Context, delta_time: Duration) {
        let mat: Matrix4<f32> = Matrix4::identity();
        let slice: Vec<u8> = mat
            .as_slice()
            .iter()
            .map(|f| f.to_ne_bytes())
            .flatten()
            .collect();
        context.draw_mesh(slice, &self.cube);
        alert("rendering frame!");
    }
}
mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    utils::set_panic_hook();
    alert("Hello, web-game!");
}
#[wasm_bindgen]
pub fn main() {
    utils::set_panic_hook();
    sukakpak::run::<TestGame>(CreateInfo {
        default_size: Vector2::new(800, 800),
        name: "test game".to_string(),
        window_id: "canvas".to_string(),
    });
}
