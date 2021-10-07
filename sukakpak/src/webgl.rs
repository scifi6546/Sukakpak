use super::{
    BackendTrait, ContextTrait, ControlFlow, CreateInfo, EventLoopTrait, GenericBindable,
    GenericDrawableTexture, MeshAsset, Timer, WindowEvent,
};
use anyhow::Result;
use image::RgbaImage;
use nalgebra::Vector2;
use std::{path::Path, time::Duration};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
pub struct EventLoop {}
impl EventLoopTrait for EventLoop {
    fn new(_: Vector2<u32>) -> Self {
        Self {}
    }
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, mut game_fn: F) -> ! {
        let mut flow = ControlFlow::Continue;
        loop {
            game_fn(WindowEvent::RunGameLogic, &mut flow);
            if flow == ControlFlow::Quit {
                panic!()
            }
        }
    }
}
pub struct Backend {
    create_info: CreateInfo,
}
impl BackendTrait for Backend {
    type EventLoop = EventLoop;
    fn new(create_info: CreateInfo, _: &Self::EventLoop) -> Self {
        Self { create_info }
    }
}
pub struct Context {
    quit: bool,
    context: WebGl2RenderingContext,
}
pub struct TimerContainer {
    /// time in ms
    time: f64,
}
impl Timer for TimerContainer {
    fn now() -> Self {
        let time = web_sys::window()
            .expect("failed to get window")
            .performance()
            .expect("failed to get performance")
            .now();
        Self { time }
    }
    fn elapsed(&self) -> Duration {
        let time = web_sys::window()
            .expect("failed to get window")
            .performance()
            .expect("failed to get performance")
            .now();
        let ms = time - self.time;
        Duration::from_micros((ms * 1000.0) as u64)
    }
}
#[derive(Debug)]
pub struct Mesh {}
#[derive(Debug)]
pub struct Framebuffer {}
#[derive(Debug)]
pub struct Texture {}
///safe becuase web browsers do not have threads
unsafe impl Send for Context {}
unsafe impl Sync for Context {}
impl ContextTrait for Context {
    type Backend = Backend;
    type Mesh = Mesh;
    type Framebuffer = Framebuffer;
    type Texture = Texture;
    type Timer = TimerContainer;
    fn new(backend: Self::Backend) -> Self {
        let canvas: HtmlCanvasElement = web_sys::window()
            .expect("failed to get window")
            .document()
            .expect("failed to get document")
            .get_element_by_id(&backend.create_info.window_id)
            .expect(&format!(
                "failed to get canvas with id: {}",
                backend.create_info.window_id
            ))
            .dyn_into()
            .expect("failed to convert to canvas");
        let context: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .expect("failed to get context")
            .expect("failed to get context")
            .dyn_into()
            .expect("failed to convert");

        Self {
            quit: false,
            context,
        }
    }
    fn begin_render(&mut self) -> Result<()> {
        Ok(())
    }
    fn finish_render(&mut self) -> Result<()> {
        Ok(())
    }
    fn build_mesh(
        &mut self,
        _: MeshAsset,
        _: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<Self::Mesh> {
        Ok(Mesh {})
    }
    fn bind_texture(
        &mut self,
        _: &mut Self::Mesh,
        _: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<()> {
        Ok(())
    }
    fn build_texture(&mut self, _: &RgbaImage) -> Result<Self::Texture> {
        Ok(Texture {})
    }
    fn draw_mesh(&mut self, _: Vec<u8>, _: &Self::Mesh) -> Result<()> {
        Ok(())
    }

    fn build_framebuffer(&mut self, _: Vector2<u32>) -> Result<Self::Framebuffer> {
        Ok(Framebuffer {})
    }
    fn bind_shader(&mut self, _: GenericBindable<Self::Framebuffer>, _: &str) -> Result<()> {
        Ok(())
    }
    fn bind_framebuffer(&mut self, _: GenericBindable<Self::Framebuffer>) -> Result<()> {
        Ok(())
    }
    fn get_screen_size(&self) -> Vector2<u32> {
        Vector2::new(100, 100)
    }
    fn load_shader<P: AsRef<Path>>(&mut self, _: P, _: &str) -> Result<()> {
        Ok(())
    }
    fn quit(&mut self) {
        self.quit = true
    }
    fn did_quit(&self) -> bool {
        self.quit
    }
    fn check_state(&mut self) {}
    fn clone(&self) -> Self {
        let quit = self.quit.clone();
        Self {
            quit,
            context: self.context.clone(),
        }
    }
}
