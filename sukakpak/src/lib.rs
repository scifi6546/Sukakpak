pub use anyhow;
use anyhow::Result;
pub use image;
use image::RgbaImage;
pub use nalgebra;
use nalgebra::Vector2;
mod events;
mod mesh;
mod vertex;
pub use mesh::{EasyMesh, Mesh as MeshAsset, Vertex as EasyMeshVertex};
use std::path::Path;
pub use vertex::{VertexComponent, VertexLayout};

pub use events::{Event, MouseButton, ScrollDelta, SemanticKeyCode};
use std::time::{Duration, SystemTime};
cfg_if::cfg_if! {
    if #[cfg(feature="backend_vulkan")]{
        mod vulkan;
        pub use vulkan::Context;
    }else{
        mod stub_backend;
        pub use stub_backend::Context;
    }
}

pub type Backend = <Context as ContextTrait>::Backend;
pub type Mesh = <Context as ContextTrait>::Mesh;
pub type Framebuffer = <Context as ContextTrait>::Framebuffer;
pub type Texture = <Context as ContextTrait>::Texture;
pub type DrawableTexture<'a> = GenericDrawableTexture<'a, Texture, Framebuffer>;
/// Represents framebuffers that can be drawn to
pub type Bindable<'a> = GenericBindable<'a, Framebuffer>;
pub struct CreateInfo {
    pub default_size: Vector2<u32>,
    pub name: String,
}
pub struct Sukakpak {}
unsafe impl Send for Sukakpak {}
pub struct EventCollector {
    events: Vec<Event>,
    quit_requested: bool,
}
impl Default for EventCollector {
    fn default() -> Self {
        Self {
            events: vec![],
            quit_requested: false,
        }
    }
}
impl EventCollector {
    pub fn push(&mut self, event: Event) {
        self.events.push(event)
    }
    pub fn pull_events(&mut self) -> Vec<Event> {
        if self.quit_requested {
            self.events.push(Event::ProgramTermination);
        }
        let r = self.events.clone();
        self.events.clear();
        return r;
    }
    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }
}
pub trait Renderable: Sized {
    fn init(context: Context) -> Self;
    fn render_frame(&mut self, events: &[Event], context: Context, delta_time: Duration);
}
impl<R: Renderable> GenericRenderable<Context> for R {
    fn init(context: Context) -> Self {
        Self::init(context)
    }
    fn render_frame(&mut self, events: &[Event], context: Context, delta_time: Duration) {
        self.render_frame(events, context, delta_time)
    }
}
/// User Provided code that provides draw calls
pub trait GenericRenderable<Ctx: ContextTrait> {
    fn init(context: Ctx) -> Self;
    fn render_frame(&mut self, events: &[Event], context: Ctx, delta_time: Duration);
}
/// Represents framebuffers that can be drawn to
pub enum GenericBindable<'a, Framebuffer> {
    UserFramebuffer(&'a Framebuffer),
    ScreenFramebuffer,
}
pub enum GenericDrawableTexture<'a, Texture, Framebuffer> {
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
    fn new(screen_size: Vector2<u32>) -> Self;
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, event: F) -> !;
}
fn generic_run<CTX, R>(create_info: CreateInfo) -> !
where
    CTX: 'static + ContextTrait,
    R: 'static + GenericRenderable<CTX>,
{
    let event_loop =
        <<CTX as ContextTrait>::Backend as BackendTrait>::EventLoop::new(create_info.default_size);
    let mut context = CTX::new(CTX::Backend::new(create_info, &event_loop));
    let mut renderer = R::init(context.clone());
    let mut system_time = SystemTime::now();
    let mut event_collector = EventCollector::default();
    event_loop.run(move |event, control_flow| {
        match event {
            WindowEvent::Event(event) => event_collector.push(event),
            WindowEvent::RunGameLogic => {
                let delta_time = system_time.elapsed().expect("failed to get time");
                context.begin_render().expect("failed  begin to render");
                renderer.render_frame(&event_collector.pull_events(), context.clone(), delta_time);
                if context.did_quit() {
                    *control_flow = ControlFlow::Quit;
                }
                context.finish_render().expect("failed to finish");
                system_time = SystemTime::now();
            }
        };
        if event_collector.quit_requested() {
            *control_flow = ControlFlow::Quit;
        }
    });
}
/// Entry point to run game. Use this to start rendering.
pub fn run<R: 'static + GenericRenderable<Context>>(create_info: CreateInfo) -> ! {
    generic_run::<Context, R>(create_info)
}

pub trait BackendTrait {
    type EventLoop: EventLoopTrait;
    fn new(create_info: CreateInfo, event_loop: &Self::EventLoop) -> Self;
}
/// Generic Graphics context. All backends implement this.
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
        texture: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<Self::Mesh>;
    /// Binds a texture.
    /// Preconditions
    /// None
    fn bind_texture(
        &mut self,
        mesh: &mut Self::Mesh,
        texture: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<()>;
    fn build_texture(&mut self, image: &RgbaImage) -> Result<Self::Texture>;
    fn draw_mesh(&mut self, push: Vec<u8>, mesh: &Self::Mesh) -> Result<()>;
    fn build_framebuffer(&mut self, resolution: Vector2<u32>) -> Result<Self::Framebuffer>;
    /// Shader being stringly typed is not ideal but better shader system is waiting
    /// on a naga translation layer for shaders
    fn bind_shader(
        &mut self,
        framebuffer: GenericBindable<Self::Framebuffer>,
        shader: &str,
    ) -> Result<()>;
    fn bind_framebuffer(&mut self, framebuffer: GenericBindable<Self::Framebuffer>) -> Result<()>;
    /// Gets screen resolution in pixels
    fn get_screen_size(&self) -> Vector2<u32>;

    fn load_shader<P: AsRef<Path>>(&mut self, path: P, shader_name: &str) -> Result<()>;
    /// quits the program once `render_frame` finishes
    fn quit(&mut self);
    ///checks if quit was called
    fn did_quit(&self) -> bool;
    /// Checks state. If state validation feature is enabled
    fn check_state(&mut self);
    /// makes copy of data, points to same base data. Must be thread safe.
    /// Otherwards must be able to make draw calls from both copies to same
    /// render surface on same thread
    fn clone(&self) -> Self;
}
