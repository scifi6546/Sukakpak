pub use anyhow;
use anyhow::Result;
mod backend;
mod events;
use backend::{Backend, VertexComponent, VertexLayout};
pub use backend::{
    BoundFramebuffer, FramebufferID as Framebuffer, MeshID as Mesh, MeshTexture,
    TextureID as Texture,
};
use events::EventCollector;
pub use events::{Event, MouseButton};
pub use image;
use image::RgbaImage;
mod mesh;
pub use backend::BackendCreateInfo as CreateInfo;
pub use mesh::{EasyMesh, Mesh as MeshAsset};
pub use nalgebra;
use nalgebra as na;
use nalgebra::Vector2;
use std::time::SystemTime;
use winit::{event::Event as WinitEvent, event_loop::ControlFlow};
pub struct Context {
    backend: Backend,
}
unsafe impl Send for Context {}
impl Context {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<R: 'static + Renderable>(create_info: CreateInfo) -> ! {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut context = Context {
            backend: Backend::new(create_info, &event_loop).expect("failed to create backend"),
        };

        let mut render = {
            let mut child = ContextChild::new(&mut context);
            R::init(&mut child)
        };

        let mut event_collector = EventCollector::new();
        let mut system_time = SystemTime::now();
        event_loop.run(move |event, _, control_flow| {
            match event {
                WinitEvent::WindowEvent { event, .. } => {
                    event_collector.push_event(event, &context.backend)
                }
                WinitEvent::MainEventsCleared => {
                    let delta_time = system_time.elapsed().expect("failed to get time");
                    match run_frame(
                        &event_collector.pull_events(),
                        &mut render,
                        &mut context,
                        delta_time.as_micros() as f32 / 1000.0,
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
    context: &mut Context,
    delta_time_ms: f32,
) -> FrameStatus {
    context
        .backend
        .begin_render()
        .expect("failed to start rendering frame");
    let mut child = ContextChild::new(context);
    renderer.render_frame(events, &mut child, delta_time_ms);
    if !child.quit {
        context
            .backend
            .finish_render()
            .expect("failed to swap framebuffer");
        FrameStatus::Continue
    } else {
        FrameStatus::Quit
    }
}

pub struct ContextChild<'a> {
    context: &'a mut Context,
    //true if quit is signaled
    quit: bool,
}

//draws meshes. Will draw on update_uniform, bind_framebuffer, or force_draw
impl<'a> ContextChild<'a> {
    fn new(context: &'a mut Context) -> Self {
        Self {
            context,
            quit: false,
        }
    }
    pub fn build_mesh(&mut self, mesh: MeshAsset, texture: MeshTexture) -> Mesh {
        Mesh {
            verticies: self
                .context
                .backend
                .allocate_verticies(mesh.verticies, mesh.vertex_layout)
                .expect("failed to allocate mesh"),
            indicies: self
                .context
                .backend
                .allocate_indicies(mesh.indices)
                .expect("failed to allocate indicies"),
            texture,
        }
    }
    pub fn build_texture(&mut self, image: &RgbaImage) -> Result<MeshTexture> {
        Ok(MeshTexture::RegularTexture(
            self.context.backend.allocate_texture(image)?,
        ))
    }
    pub fn draw_mesh(&mut self, push: &[u8], mesh: &Mesh) -> Result<()> {
        self.context.backend.draw_mesh(push, mesh)
    }
    pub fn build_framebuffer(&mut self, resolution: na::Vector2<u32>) -> Result<Framebuffer> {
        self.context.backend.build_framebuffer(resolution)
    }
    /// Shader being stringly typed is not ideal but better shader system is waiting
    /// on a naga translation layer for shaders
    pub fn bind_shader(&mut self, framebuffer: &BoundFramebuffer, shader: &str) -> Result<()> {
        self.context.backend.bind_shader(framebuffer, shader)
    }
    pub fn bind_framebuffer(&mut self, bound_framebuffer: &BoundFramebuffer) -> Result<()> {
        self.context.backend.bind_framebuffer(bound_framebuffer)
    }
    pub fn get_screen_size(&self) -> Vector2<u32> {
        self.context.backend.get_screen_size()
    }
    pub fn update_uniform(&mut self) {
        todo!("update uniform")
    }
    pub fn force_draw(&mut self) {
        todo!("force draw")
    }
    /// quits the program once `render_frame` finishes
    pub fn quit(&mut self) {
        self.quit = true;
    }
}
/// User Provided code that provides draw calls
pub trait Renderable {
    fn init<'a>(context: &mut ContextChild<'a>) -> Self;
    fn render_frame<'a>(
        &mut self,
        events: &[Event],
        context: &mut ContextChild<'a>,
        delta_time_ms: f32,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Matrix4, Vector2};

    struct EmptyRenderable {}
    impl Renderable for EmptyRenderable {
        fn init<'a>(_context: &mut ContextChild<'a>) -> Self {
            Self {}
        }
        fn render_frame<'a>(
            &mut self,
            _events: &[Event],
            context: &mut ContextChild<'a>,
            _delta_time: f32,
        ) {
            context.quit();
        }
    }
    struct TriangleRenderable {
        num_frames: usize,
        triangle: Mesh,
        #[allow(dead_code)]
        texture: MeshTexture,
    }
    impl Renderable for TriangleRenderable {
        fn init<'a>(context: &mut ContextChild<'a>) -> Self {
            let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 0, 0, 0]));
            let texture = context
                .build_texture(&image)
                .expect("failed to create image");
            let triangle = context.build_mesh(MeshAsset::new_triangle(), texture);
            Self {
                triangle,
                num_frames: 0,
                texture,
            }
        }
        fn render_frame<'a>(
            &mut self,
            _events: &[Event],
            context: &mut ContextChild<'a>,
            _dt: f32,
        ) {
            if self.num_frames <= 10_000 {
                let mat = Matrix4::identity();
                let mat_ptr = mat.as_ptr() as *const u8;
                let push = unsafe { std::slice::from_raw_parts(mat_ptr, 16 * 4) };
                context
                    .draw_mesh(push, &self.triangle)
                    .expect("failed to draw triangle");
                self.num_frames += 1;
            } else {
                context.quit();
            }
        }
    }
    #[test]
    fn draw_triangle() {
        //should start and stop without issue
        Context::new::<TriangleRenderable>(CreateInfo {
            default_size: Vector2::new(800, 800),
            name: String::from("Draw Triangle"),
        });
    }
}
