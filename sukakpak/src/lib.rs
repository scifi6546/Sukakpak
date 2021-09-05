pub use anyhow;
use anyhow::Result;
mod backend;
mod events;
use backend::Backend;
pub use backend::{
    BoundFramebuffer, FramebufferID as Framebuffer, MeshID as Mesh, MeshTexture,
    TextureID as Texture, VertexComponent, VertexLayout,
};
use events::EventCollector;
pub use events::{Event, MouseButton};
pub use image;
use image::RgbaImage;
mod mesh;
pub use backend::BackendCreateInfo as CreateInfo;
pub use mesh::{EasyMesh, Mesh as MeshAsset, Vertex as EasyMeshVertex};
pub use nalgebra;
use nalgebra as na;
use nalgebra::Vector2;
use std::{cell::RefCell, path::Path, rc::Rc, time::Duration, time::SystemTime};
use winit::{event::Event as WinitEvent, event_loop::ControlFlow};
pub struct Sukakpak {}
unsafe impl Send for Sukakpak {}
unsafe impl Send for Context {}
impl Sukakpak {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<R: 'static + Renderable>(create_info: CreateInfo) -> ! {
        let event_loop = winit::event_loop::EventLoop::new();

        let context = Rc::new(RefCell::new(Context::new(
            Backend::new(create_info, &event_loop).expect("failed to create backend"),
        )));
        let mut renderer = R::init(Rc::clone(&context));

        let mut event_collector = EventCollector::new();
        let mut system_time = SystemTime::now();

        event_loop.run(move |event, _, control_flow| {
            match event {
                WinitEvent::WindowEvent { event, .. } => {
                    let ctx_borrow = context.borrow_mut();
                    event_collector.push_event(event, &ctx_borrow.backend)
                }
                WinitEvent::MainEventsCleared => {
                    let delta_time = system_time.elapsed().expect("failed to get time");
                    match run_frame(
                        &event_collector.pull_events(),
                        &mut renderer,
                        Rc::clone(&context),
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
    context: Rc<RefCell<Context>>,
    delta_time: Duration,
) -> FrameStatus {
    {
        let mut ctx_borrow = context.borrow_mut();
        ctx_borrow
            .backend
            .begin_render()
            .expect("failed to start rendering frame");
    }

    renderer.render_frame(events, Rc::clone(&context), delta_time);
    let mut ctx_borrow = context.borrow_mut();
    if !ctx_borrow.quit {
        ctx_borrow
            .finish_render()
            .expect("failed to finish rendering");
        FrameStatus::Continue
    } else {
        FrameStatus::Quit
    }
}

pub struct Context {
    backend: Backend,
    //true if quit is signaled
    quit: bool,
}

//draws meshes. Will draw on update_uniform, bind_framebuffer, or force_draw
impl Context {
    fn new(backend: Backend) -> Self {
        Self {
            backend,
            quit: false,
        }
    }
    /// Does steps for finshing rendering
    fn finish_render(&mut self) -> Result<()> {
        self.check_state();
        self.backend.finish_render()?;
        self.check_state();
        self.backend.collect_garbage()
    }
    pub fn build_mesh(&mut self, mesh: MeshAsset, texture: MeshTexture) -> Result<Mesh> {
        self.check_state();
        let mesh =
            self.backend
                .build_mesh(mesh.vertices, mesh.vertex_layout, mesh.indices, texture)?;
        self.check_state();
        Ok(mesh)
    }
    /// Binds a texture.
    /// Preconditions
    /// None
    pub fn bind_texture(&mut self, mesh: &mut Mesh, texture: &MeshTexture) -> Result<()> {
        self.check_state();
        self.backend.bind_texture(mesh, texture.clone())?;
        self.check_state();
        Ok(())
    }
    /// Deletes Mesh. Mesh not be used in current draw call.
    pub fn delete_mesh(&mut self, mesh: Mesh) -> Result<()> {
        self.check_state();
        self.backend.free_mesh(&mesh)?;
        self.check_state();
        Ok(())
    }
    pub fn build_texture(&mut self, image: &RgbaImage) -> Result<MeshTexture> {
        self.check_state();
        let mesh = MeshTexture::RegularTexture(self.backend.allocate_texture(image)?);
        self.check_state();

        Ok(mesh)
    }
    /// Deletes Texture. Texture must not be used in current draw call.
    pub fn delete_texture(&mut self, tex: MeshTexture) -> Result<()> {
        self.check_state();
        match tex {
            MeshTexture::RegularTexture(texture) => self
                .backend
                .free_texture(MeshTexture::RegularTexture(texture)),
            MeshTexture::Framebuffer(_fb) => todo!("free framebuffer"),
        }
    }
    pub fn draw_mesh(&mut self, push: Vec<u8>, mesh: &Mesh) -> Result<()> {
        self.check_state();
        self.backend.draw_mesh(push, mesh)?;
        self.check_state();
        Ok(())
    }
    pub fn build_framebuffer(&mut self, resolution: na::Vector2<u32>) -> Result<Framebuffer> {
        self.backend.build_framebuffer(resolution)
    }
    /// Shader being stringly typed is not ideal but better shader system is waiting
    /// on a naga translation layer for shaders
    pub fn bind_shader(&mut self, framebuffer: &BoundFramebuffer, shader: &str) -> Result<()> {
        self.check_state();
        self.backend.bind_shader(framebuffer, shader)?;
        self.check_state();
        Ok(())
    }
    pub fn bind_framebuffer(&mut self, bound_framebuffer: &BoundFramebuffer) -> Result<()> {
        self.check_state();
        self.backend.bind_framebuffer(bound_framebuffer)?;
        self.check_state();
        Ok(())
    }
    pub fn get_screen_size(&self) -> Vector2<u32> {
        self.backend.get_screen_size()
    }
    pub fn update_uniform(&mut self) {
        todo!("update uniform")
    }
    pub fn force_draw(&mut self) {
        todo!("force draw")
    }
    pub fn load_shader<P: AsRef<Path>>(&mut self, path: P, shader_name: &str) -> Result<()> {
        self.check_state();
        self.backend.load_shader(path, shader_name)?;
        self.check_state();
        Ok(())
    }
    /// quits the program once `render_frame` finishes
    pub fn quit(&mut self) {
        self.quit = true;
    }
    /// Checks state. If state validation feature is enabled
    fn check_state(&mut self) {
        #[cfg(feature = "state_validation")]
        self.backend.check_state();
    }
}
/// User Provided code that provides draw calls
pub trait Renderable {
    fn init(context: Rc<RefCell<Context>>) -> Self;
    fn render_frame(
        &mut self,
        events: &[Event],
        context: Rc<RefCell<Context>>,
        delta_time: Duration,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Matrix4, Vector2};

    struct EmptyRenderable {}
    impl Renderable for EmptyRenderable {
        fn init<'a>(_context: Rc<RefCell<Context>>) -> Self {
            Self {}
        }
        fn render_frame<'a>(
            &mut self,
            _events: &[Event],
            context: Rc<RefCell<Context>>,
            _delta_time: Duration,
        ) {
            let mut ctx_borrow = context.borrow_mut();
            ctx_borrow.quit();
        }
    }
    struct TriangleRenderable {
        num_frames: usize,
        triangle: Mesh,
        #[allow(dead_code)]
        texture: MeshTexture,
    }
    impl Renderable for TriangleRenderable {
        fn init<'a>(context: Rc<RefCell<Context>>) -> Self {
            let mut ctx_borrow = context.borrow_mut();
            let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 0, 0, 0]));
            let texture = ctx_borrow
                .build_texture(&image)
                .expect("failed to create image");
            let triangle = ctx_borrow.build_mesh(MeshAsset::new_triangle(), texture);
            Self {
                triangle,
                num_frames: 0,
                texture,
            }
        }
        fn render_frame<'a>(
            &mut self,
            _events: &[Event],
            context: Rc<RefCell<Context>>,
            _dt: Duration,
        ) {
            let mut ctx_borrow = context.borrow_mut();
            if self.num_frames <= 10_000 {
                let mat = Matrix4::<f32>::identity();
                ctx_borrow
                    .draw_mesh(
                        mat.as_slice()
                            .iter()
                            .map(|f| f.to_ne_bytes())
                            .flatten()
                            .collect(),
                        &self.triangle,
                    )
                    .expect("failed to draw triangle");
                self.num_frames += 1;
            } else {
                ctx_borrow.quit();
            }
        }
    }
    #[test]
    fn draw_triangle() {
        //should start and stop without issue
        Sukakpak::new::<TriangleRenderable>(CreateInfo {
            default_size: Vector2::new(800, 800),
            name: String::from("Draw Triangle"),
        });
    }
}
