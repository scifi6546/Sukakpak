mod backend;
mod event_loop;

use super::{
    BackendTrait, ContextTrait, ControlFlow, CreateInfo, EventLoopTrait, GenericBindable,
    GenericDrawableTexture, MeshAsset, Timer, VertexComponent, WindowEvent,
};
use anyhow::{bail, Result};
use ass_wgl::Shader;
use backend::Backend;
pub use backend::{Framebuffer, MeshIndex, TextureIndex};
pub use event_loop::EventLoop;
use generational_arena::{Arena, Index as ArenaIndex};
use image::RgbaImage;
use log::{info, Level};
use nalgebra::Vector2;
use std::{cell::RefCell, collections::HashMap, mem::size_of, path::Path, rc::Rc, time::Duration};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject as VAO,
};
pub struct CreateBackend {
    create_info: CreateInfo,
}
impl BackendTrait for CreateBackend {
    type EventLoop = EventLoop;
    fn new(create_info: CreateInfo, _: &Self::EventLoop) -> Self {
        Self { create_info }
    }
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
/// For now only supporting uniforms with a 4x4 matrix
pub struct Context {
    backend: Rc<RefCell<Backend>>,
}
///safe becuase web browsers do not have threads
unsafe impl Send for Context {}
unsafe impl Sync for Context {}
impl ContextTrait for Context {
    type Backend = CreateBackend;
    type Mesh = MeshIndex;
    type Framebuffer = Framebuffer;
    type Texture = TextureIndex;
    type Timer = TimerContainer;
    fn new(backend: Self::Backend) -> Self {
        let backend = Backend::new(backend);
        Self {
            backend: Rc::new(RefCell::new(backend)),
        }
    }
    fn begin_render(&mut self) -> Result<()> {
        self.backend.borrow_mut().begin_render()
    }
    fn finish_render(&mut self) -> Result<()> {
        Ok(())
    }

    fn build_mesh(
        &mut self,
        mesh: MeshAsset,
        texture: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<Self::Mesh> {
        self.backend.borrow_mut().build_mesh(mesh, texture)
    }
    fn bind_texture(
        &mut self,
        mesh: &mut Self::Mesh,
        texture: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<()> {
        self.backend.borrow_mut().bind_texture(mesh, texture)
    }
    fn build_texture(&mut self, image: &RgbaImage) -> Result<Self::Texture> {
        self.backend.borrow_mut().build_texture(image)
    }
    fn draw_mesh(&mut self, push_data: Vec<u8>, mesh_index: &Self::Mesh) -> Result<()> {
        self.backend.borrow_mut().draw_mesh(push_data, mesh_index)
    }

    fn build_framebuffer(&mut self, dimensions: Vector2<u32>) -> Result<Self::Framebuffer> {
        self.backend.borrow_mut().build_framebuffer(dimensions)
    }
    fn bind_shader(
        &mut self,
        framebuffer: GenericBindable<Self::Framebuffer>,
        shader_name: &str,
    ) -> Result<()> {
        self.backend
            .borrow_mut()
            .bind_shader(framebuffer, shader_name)
    }
    fn bind_framebuffer(&mut self, framebuffer: GenericBindable<Self::Framebuffer>) -> Result<()> {
        self.backend.borrow_mut().bind_framebuffer(framebuffer)
    }
    fn get_screen_size(&self) -> Vector2<u32> {
        self.backend.borrow_mut().get_screen_size()
    }
    fn load_shader(&mut self, shader_text: &str, name: &str) -> Result<()> {
        self.backend.borrow_mut().load_shader(shader_text, name)
    }
    fn quit(&mut self) {
        self.backend.borrow_mut().quit()
    }
    fn did_quit(&self) -> bool {
        self.backend.borrow_mut().did_quit()
    }
    fn check_state(&mut self) {
        self.backend.borrow_mut().check_state();
    }
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
        }
    }
}
