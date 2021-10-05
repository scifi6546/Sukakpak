use super::{
    BackendTrait, ContextTrait, ControlFlow, CreateInfo, EventLoopTrait, GenericBindable,
    GenericDrawableTexture, MeshAsset, WindowEvent,
};
use anyhow::Result;
use image::RgbaImage;
use nalgebra::Vector2;
use std::path::Path;
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
    quit: bool,
}
impl ContextTrait for Context {
    type Backend = Backend;
    type Mesh = ();
    type Framebuffer = ();
    type Texture = ();
    fn new(_: Self::Backend) -> Self {
        Self { quit: false }
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
        Ok(())
    }
    fn bind_texture(
        &mut self,
        _: &mut Self::Mesh,
        _: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<()> {
        Ok(())
    }
    fn build_texture(&mut self, _: &RgbaImage) -> Result<Self::Texture> {
        Ok(())
    }
    fn draw_mesh(&mut self, _: Vec<u8>, _: &Self::Mesh) -> Result<()> {
        Ok(())
    }

    fn build_framebuffer(&mut self, _: Vector2<u32>) -> Result<Self::Framebuffer> {
        Ok(())
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
        Self { quit: self.quit }
    }
}
