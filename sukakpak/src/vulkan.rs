mod backend;
pub mod events;
use anyhow::Result;
use backend::{Backend, BoundFramebuffer, FramebufferID, MeshID, TextureID};

pub use super::mesh::{EasyMesh, Mesh as MeshAsset, Vertex as EasyMeshVertex};
pub use backend::{BackendCreateInfo as CreateInfo, MeshTexture, VertexComponent, VertexLayout};
use events::{Event, EventCollector};
use image;
use image::RgbaImage;
use nalgebra;
use nalgebra as na;
use nalgebra::Vector2;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
    time::SystemTime,
};
use winit::{event::Event as WinitEvent, event_loop::ControlFlow};
unsafe impl Send for Mesh {}
pub struct Mesh {
    mesh: MeshID,
    backend: Arc<Mutex<Backend>>,
}
impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh").field("mesh", &self.mesh).finish()
    }
}
impl Drop for Mesh {
    fn drop(&mut self) {
        self.backend
            .lock()
            .expect("failed to get lock")
            .free_mesh(&self.mesh)
            .expect("failed to free mesh");
    }
}
unsafe impl Send for Texture {}
pub struct Texture {
    texture: TextureID,
    backend: Arc<Mutex<Backend>>,
}
impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("Texture", &self.texture)
            .finish()
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        self.backend
            .lock()
            .expect("failed to get lock")
            .free_texture(MeshTexture::RegularTexture(self.texture))
            .expect("failed to free texture");
    }
}
unsafe impl Send for Framebuffer {}
pub struct Framebuffer {
    framebuffer: FramebufferID,
    backend: Arc<Mutex<Backend>>,
}
/// Represents framebuffers that can be drawn to
pub enum Bindable<'a> {
    UserFramebuffer(&'a Framebuffer),
    ScreenFramebuffer,
}
impl From<Bindable<'_>> for BoundFramebuffer {
    fn from(bind: Bindable) -> Self {
        match bind {
            Bindable::UserFramebuffer(fb) => Self::UserFramebuffer(fb.framebuffer),
            Bindable::ScreenFramebuffer => Self::ScreenFramebuffer,
        }
    }
}
impl Drop for Framebuffer {
    fn drop(&mut self) {
        self.backend
            .lock()
            .expect("failed to get lock")
            .free_texture(MeshTexture::Framebuffer(self.framebuffer))
            .expect("failed to free texture");
    }
}
pub enum DrawableTexture<'a> {
    Texture(&'a Texture),
    Framebuffer(&'a Framebuffer),
}
impl From<DrawableTexture<'_>> for MeshTexture {
    fn from(tex: DrawableTexture) -> Self {
        match tex {
            DrawableTexture::Texture(tex) => Self::RegularTexture(tex.texture),
            DrawableTexture::Framebuffer(fb) => Self::Framebuffer(fb.framebuffer),
        }
    }
}

pub struct Sukakpak {}
unsafe impl Send for Sukakpak {}
unsafe impl Send for Context {}
impl Sukakpak {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<R: 'static + Renderable>(create_info: CreateInfo) -> ! {
        let event_loop = winit::event_loop::EventLoop::new();

        let context =
            Context::new(Backend::new(create_info, &event_loop).expect("failed to create backend"));
        let mut renderer = R::init(context.clone());

        let mut event_collector = EventCollector::new();
        let mut system_time = SystemTime::now();

        event_loop.run(move |event, _, control_flow| {
            match event {
                WinitEvent::WindowEvent { event, .. } => {
                    event_collector.push_event(event, &context.backend.lock().unwrap())
                }
                WinitEvent::MainEventsCleared => {
                    let delta_time = system_time.elapsed().expect("failed to get time");
                    match run_frame(
                        &event_collector.pull_events(),
                        &mut renderer,
                        context.clone(),
                        delta_time,
                    ) {
                        FrameStatus::Quit => *control_flow = ControlFlow::Exit,
                        FrameStatus::Continue => (),
                    };
                    system_time = SystemTime::now();
                }
                _ => (),
            }

            if event_collector.quit_done() {
                *control_flow = ControlFlow::Exit
            }
        });
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrameStatus {
    Continue,
    Quit,
}
fn run_frame<R: Renderable>(
    events: &[Event],
    renderer: &mut R,
    mut context: Context,
    delta_time: Duration,
) -> FrameStatus {
    {
        context
            .backend
            .lock()
            .expect("failed to get lock")
            .begin_render()
            .expect("failed to start rendering frame");
    }

    renderer.render_frame(events, context.clone(), delta_time);

    if *context.quit.lock().expect("failed to get lock") == false {
        context.finish_render().expect("failed to finish rendering");
        FrameStatus::Continue
    } else {
        FrameStatus::Quit
    }
}

#[derive(Clone)]
pub struct Context {
    backend: Arc<Mutex<Backend>>,
    /// true if quit is signaled
    quit: Arc<Mutex<bool>>,
}

//draws meshes. Will draw on update_uniform, bind_framebuffer, or force_draw
impl Context {
    fn new(backend: Backend) -> Self {
        Self {
            backend: Arc::new(Mutex::new(backend)),
            quit: Arc::new(Mutex::new(false)),
        }
    }
    /// Does steps for finshing rendering
    fn finish_render(&mut self) -> Result<()> {
        self.check_state();
        {
            let mut backend_lock = self.backend.lock().expect("failed to get lock");
            backend_lock.finish_render()?;
            backend_lock.collect_garbage()?;
        }
        self.check_state();
        Ok(())
    }
    pub fn build_mesh(&mut self, mesh: MeshAsset, texture: DrawableTexture) -> Result<Mesh> {
        self.check_state();
        let mesh = self
            .backend
            .lock()
            .expect("failed to get lock")
            .build_mesh(
                mesh.vertices,
                mesh.vertex_layout,
                mesh.indices,
                texture.into(),
            )?;

        self.check_state();
        Ok(Mesh {
            mesh,
            backend: self.backend.clone(),
        })
    }
    /// Binds a texture.
    /// Preconditions
    /// None
    pub fn bind_texture(&mut self, mesh: &mut Mesh, texture: DrawableTexture) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .bind_texture(&mut mesh.mesh, texture.into())?;
        self.check_state();
        Ok(())
    }
    pub fn build_texture(&mut self, image: &RgbaImage) -> Result<Texture> {
        self.check_state();
        let texture = self
            .backend
            .lock()
            .expect("failed to get lock")
            .allocate_texture(image)?;

        self.check_state();

        Ok(Texture {
            texture,
            backend: self.backend.clone(),
        })
    }
    /// Deletes Texture. Texture must not be used in current draw call.
    pub fn delete_texture(&mut self, tex: MeshTexture) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .free_texture(tex)?;
        self.check_state();
        Ok(())
    }
    pub fn draw_mesh(&mut self, push: Vec<u8>, mesh: &Mesh) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .draw_mesh(push, &mesh.mesh)?;
        self.check_state();
        Ok(())
    }
    pub fn build_framebuffer(&mut self, resolution: na::Vector2<u32>) -> Result<Framebuffer> {
        let framebuffer = self
            .backend
            .lock()
            .expect("failed to get lock")
            .build_framebuffer(resolution)
            .expect("failed to build framebuffer");
        Ok(Framebuffer {
            framebuffer,
            backend: self.backend.clone(),
        })
    }
    /// Shader being stringly typed is not ideal but better shader system is waiting
    /// on a naga translation layer for shaders
    pub fn bind_shader(&mut self, framebuffer: Bindable, shader: &str) -> Result<()> {
        self.check_state();

        self.backend
            .lock()
            .expect("failed to get lock")
            .bind_shader(&framebuffer.into(), shader)?;
        self.check_state();
        Ok(())
    }

    pub fn bind_framebuffer(&mut self, framebuffer: Bindable) -> Result<()> {
        self.backend
            .lock()
            .unwrap()
            .bind_framebuffer(&framebuffer.into())?;
        self.check_state();
        Ok(())
    }
    /// Gets screen resolution in pixels
    pub fn get_screen_size(&self) -> Vector2<u32> {
        self.backend
            .lock()
            .expect("failed to get lock")
            .get_screen_size()
    }
    pub fn load_shader<P: AsRef<Path>>(&mut self, path: P, shader_name: &str) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .load_shader(path, shader_name)?;
        self.check_state();
        Ok(())
    }
    /// quits the program once `render_frame` finishes
    pub fn quit(&mut self) {
        *self.quit.lock().unwrap() = true;
    }
    /// Checks state. If state validation feature is enabled
    fn check_state(&mut self) {
        #[cfg(feature = "state_validation")]
        self.backend
            .lock()
            .expect("failed to get lock")
            .check_state();
    }
}
/// User Provided code that provides draw calls
pub trait Renderable {
    fn init(context: Context) -> Self;
    fn render_frame(&mut self, events: &[Event], context: Context, delta_time: Duration);
}
