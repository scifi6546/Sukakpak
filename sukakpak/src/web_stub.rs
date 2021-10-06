use super::{
    BackendTrait, ContextTrait, ControlFlow, CreateInfo, EventLoopTrait, GenericBindable,
    GenericDrawableTexture, MeshAsset, Timer, WindowEvent,
};
use anyhow::Result;
use image::RgbaImage;
use nalgebra::Vector2;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use web_sys::DateTimeValue;
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
pub struct Backend {}
impl BackendTrait for Backend {
    type EventLoop = EventLoop;
    fn new(_: CreateInfo, _: &Self::EventLoop) -> Self {
        Self {}
    }
}
pub struct Context {
    quit: Arc<Mutex<bool>>,
}
pub struct TimerContainer {
    time: DateTimeValue,
}
impl Timer for TimerContainer {
    fn now() -> Self {
        Self {
            time: DateTimeValue::new(),
        }
    }
    fn elapsed(&self) -> Duration {
        let new_time = DateTimeValue::new();
        todo!()
    }
}
#[derive(Debug)]
pub struct Mesh {}
#[derive(Debug)]
pub struct Framebuffer {}
#[derive(Debug)]
pub struct Texture {}
impl ContextTrait for Context {
    type Backend = Backend;
    type Mesh = Mesh;
    type Framebuffer = Framebuffer;
    type Texture = Texture;
    type Timer = TimerContainer;
    fn new(_: Self::Backend) -> Self {
        Self {
            quit: Arc::new(Mutex::new(false)),
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
        *self.quit.lock().expect("failed to get lock") = true
    }
    fn did_quit(&self) -> bool {
        *self.quit.lock().expect("failed to get lock")
    }
    fn check_state(&mut self) {}
    fn clone(&self) -> Self {
        let quit = self.quit.clone();
        Self { quit }
    }
}
