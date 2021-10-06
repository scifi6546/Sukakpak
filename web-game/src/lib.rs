use std::time::Duration;
use sukakpak::{nalgebra::Vector2, Context, CreateInfo, Event, Renderable};
pub struct TestGame {}
impl Renderable for TestGame {
    fn init(context: Context) -> Self {
        Self {}
    }
    fn render_frame(&mut self, events: &[Event], context: Context, delta_time: Duration) {
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
    });
}
