mod backend;
pub mod events;
use anyhow::Result;
use backend::{Backend, BoundFramebuffer, FramebufferID, MeshID, TextureID};

use super::{
    mesh::Mesh as MeshAsset, CreateInfo, Event, MouseButton, ScrollDelta, SemanticKeyCode, Timer,
};
use super::{VertexComponent, VertexLayout};
pub use backend::MeshTexture;
use image;
use image::RgbaImage;
use nalgebra;

use nalgebra::Vector2;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub struct TimerContainer {
    instant: Instant,
}
impl Timer for TimerContainer {
    fn now() -> Self {
        Self {
            instant: Instant::now(),
        }
    }
    fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }
}
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
impl From<super::GenericBindable<'_, Framebuffer>> for BoundFramebuffer {
    fn from(bind: super::GenericBindable<'_, Framebuffer>) -> Self {
        match bind {
            super::GenericBindable::UserFramebuffer(fb) => Self::UserFramebuffer(fb.framebuffer),
            super::GenericBindable::ScreenFramebuffer => Self::ScreenFramebuffer,
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

impl From<super::GenericDrawableTexture<'_, Texture, Framebuffer>> for MeshTexture {
    fn from(tex: super::GenericDrawableTexture<'_, Texture, Framebuffer>) -> Self {
        match tex {
            super::GenericDrawableTexture::Texture(tex) => Self::RegularTexture(tex.texture),
            super::GenericDrawableTexture::Framebuffer(fb) => Self::Framebuffer(fb.framebuffer),
        }
    }
}
pub struct BackendArc(Arc<Mutex<Backend>>);
pub struct Context {
    backend: Arc<Mutex<Backend>>,
    /// true if quit is signaled
    quit: Arc<Mutex<bool>>,
}
impl super::BackendTrait for BackendArc {
    type EventLoop = events::WinitEventLoopAdaptor;
    fn new(create_info: super::CreateInfo, event_loop: &Self::EventLoop) -> Self {
        let backend = Backend::new(create_info, event_loop.event_loop())
            .expect("failed to initialize backend");
        Self(Arc::new(Mutex::new(backend)))
    }
}
impl super::ContextTrait for Context {
    type Backend = BackendArc;
    type Mesh = Mesh;
    type Framebuffer = Framebuffer;
    type Texture = Texture;
    type Timer = TimerContainer;
    fn new(backend: BackendArc) -> Self {
        Self {
            backend: backend.0,
            quit: Arc::new(Mutex::new(false)),
        }
    }
    fn begin_render(&mut self) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .begin_render()
    }
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
    fn build_mesh(
        &mut self,
        mesh: MeshAsset,
        texture: super::GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<Self::Mesh> {
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
    fn bind_texture(
        &mut self,
        mesh: &mut Self::Mesh,
        texture: super::GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .bind_texture(&mut mesh.mesh, texture.into())?;
        self.check_state();
        Ok(())
    }
    fn build_texture(&mut self, image: &RgbaImage) -> Result<Self::Texture> {
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
    fn draw_mesh(&mut self, push: Vec<u8>, mesh: &Self::Mesh) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .draw_mesh(push, &mesh.mesh)?;
        self.check_state();
        Ok(())
    }
    fn build_framebuffer(&mut self, resolution: Vector2<u32>) -> Result<Framebuffer> {
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
    fn bind_shader(
        &mut self,
        framebuffer: super::GenericBindable<Self::Framebuffer>,
        shader: &str,
    ) -> Result<()> {
        self.check_state();

        self.backend
            .lock()
            .expect("failed to get lock")
            .bind_shader(&framebuffer.into(), shader)?;
        self.check_state();
        Ok(())
    }
    fn bind_framebuffer(
        &mut self,
        framebuffer: super::GenericBindable<Self::Framebuffer>,
    ) -> Result<()> {
        self.backend
            .lock()
            .unwrap()
            .bind_framebuffer(&framebuffer.into())?;
        self.check_state();
        Ok(())
    }
    fn get_screen_size(&self) -> Vector2<u32> {
        self.backend
            .lock()
            .expect("failed to get lock")
            .get_screen_size()
    }
    fn load_shader<P: AsRef<Path>>(&mut self, path: P, shader_name: &str) -> Result<()> {
        self.check_state();
        self.backend
            .lock()
            .expect("failed to get lock")
            .load_shader(path, shader_name)?;
        self.check_state();
        Ok(())
    }
    fn quit(&mut self) {
        *self.quit.lock().unwrap() = true;
    }
    fn did_quit(&self) -> bool {
        *self.quit.lock().unwrap()
    }
    fn check_state(&mut self) {
        #[cfg(feature = "state_validation")]
        self.backend
            .lock()
            .expect("failed to get lock")
            .check_state();
    }
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            quit: self.quit.clone(),
        }
    }
}
