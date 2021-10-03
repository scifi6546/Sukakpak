pub use anyhow;
use anyhow::Result;
pub use image;
use image::RgbaImage;
pub use nalgebra;
use nalgebra::Vector2;
mod mesh;
pub use mesh::{EasyMesh, Mesh as MeshAsset, Vertex as EasyMeshVertex};
use std::path::Path;
mod vulkan;
use std::time::{Duration, SystemTime};
pub use vulkan::{
    events::{Event, MouseButton},
    Bindable as TBindable, CreateInfo, DrawableTexture as TDrawableTexture, Framebuffer,
    MeshTexture, Texture, VertexComponent, VertexLayout,
};
pub struct Sukakpak {}
unsafe impl Send for Sukakpak {}
pub struct EventCollector {}
impl Default for EventCollector {
    fn default() -> Self {
        todo!()
    }
}
impl EventCollector {
    pub fn push<CTX: ContextTrait>(&mut self, event: Event, ctx: CTX) {
        todo!()
    }
    pub fn pull_events(&self) -> &[Event] {
        todo!()
    }
    pub fn quit_requested(&self) -> bool {
        todo!()
    }
}
/// User Provided code that provides draw calls
pub trait Renderable<Ctx: ContextTrait> {
    fn init(context: Ctx) -> Self;
    fn render_frame(&mut self, events: &[Event], context: Ctx, delta_time: Duration);
}
pub enum Bindable<'a, Framebuffer> {
    UserFramebuffer(&'a Framebuffer),
    ScreenFramebuffer,
}
pub enum DrawableTexture<'a, Texture, Framebuffer> {
    Texture(&'a Texture),
    Framebuffer(&'a Framebuffer),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFlow {
    Continue,
    Quit,
}
pub enum WindowEvent {
    Event(Event),
    RunGameLogic,
}
pub trait EventLoopTrait {
    fn new() -> Self;
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, event: F) -> !;
}
impl Sukakpak {
    pub fn new<CTX, R>(create_info: CreateInfo) -> !
    where
        CTX: 'static + ContextTrait,
        R: 'static + Renderable<CTX>,
    {
        let event_loop = <<CTX as ContextTrait>::Backend as BackendTrait>::EventLoop::new();
        let mut context = CTX::new(CTX::Backend::new(create_info, &event_loop));
        let mut renderer = R::init(context.clone());
        let system_time = SystemTime::now();
        let mut event_collector = EventCollector::default();
        event_loop.run(move |event, control_flow| {
            match event {
                WindowEvent::Event(event) => event_collector.push(event, context.clone()),
                WindowEvent::RunGameLogic => {
                    let delta_time = system_time.elapsed().expect("failed to get time");
                    context.begin_render().expect("failed  begin to render");
                    renderer.render_frame(
                        event_collector.pull_events(),
                        context.clone(),
                        delta_time,
                    );
                    if context.did_quit() {
                        *control_flow = ControlFlow::Quit;
                    }
                }
            };
            if event_collector.quit_requested() {
                *control_flow = ControlFlow::Quit;
            }
        });
    }
}
pub trait BackendTrait {
    type EventLoop: EventLoopTrait;
    fn new(create_info: CreateInfo, event_loop: &Self::EventLoop) -> Self;
}
pub trait ContextTrait: Send {
    /// backend data storing startup state
    type Backend: BackendTrait;
    /// Stores runtime mesh data. Bound texture is saved along side
    /// mesh so that texture data can only be freed once .drop is called
    /// on *both* bound texture and all meshes that bind the texture
    type Mesh;
    /// Stores runtime  framebuffer data. calling .drop on framebuffer will
    /// free the data
    type Framebuffer;
    /// Stores runtime texture data. Texture data will only be freed once  
    /// .drop is called on *both* texture and all meshes that bind the texture
    type Texture;
    fn new(backend: Self::Backend) -> Self;
    /// does steps for starting rendering
    fn begin_render(&mut self) -> Result<()>;
    /// Does steps for finshing rendering
    fn finish_render(&mut self) -> Result<()>;
    fn build_mesh(
        &mut self,
        mesh: MeshAsset,
        texture: DrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<Self::Mesh>;
    /// Binds a texture.
    /// Preconditions
    /// None
    fn bind_texture(
        &mut self,
        mesh: &mut Self::Mesh,
        texture: DrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<()>;
    fn build_texture(&mut self, image: &RgbaImage) -> Result<Self::Texture>;
    fn draw_mesh(&mut self, push: Vec<u8>, mesh: &Self::Mesh) -> Result<()>;
    fn build_framebuffer(&mut self, resolution: Vector2<u32>) -> Result<Framebuffer>;
    fn bind_shader(&mut self, framebuffer: Bindable<Self::Framebuffer>, shader: &str)
        -> Result<()>;
    fn bind_framebuffer(&mut self, framebuffer: Bindable<Self::Framebuffer>) -> Result<()>;
    /// Gets screen resolution in pixels
    fn get_screen_size(&self) -> Vector2<u32>;

    fn load_shader<P: AsRef<Path>>(&mut self, path: P, shader_name: &str) -> Result<()>;
    /// quits the program once `render_frame` finishes
    fn quit(&mut self);
    //checks if quit was called
    fn did_quit(&self) -> bool;
    /// Checks state. If state validation feature is enabled
    fn check_state(&mut self);
    /// makes copy of data, points to same base data. Must be thread safe.
    /// Otherwards must be able to make draw calls from both copies to same
    /// render surface on same thread
    fn clone(&self) -> Self;
}
