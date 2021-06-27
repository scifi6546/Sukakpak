use anyhow::Result;
mod backend;
use backend::{Backend, VertexLayout};
pub use backend::{
    BoundFramebuffer, FramebufferID as Framebuffer, MeshID as Mesh, MeshTexture,
    TextureID as Texture,
};
use image::RgbaImage;
mod mesh;
pub use backend::BackendCreateInfo as CreateInfo;
pub use mesh::{EasyMesh, Mesh as MeshAsset};
pub use nalgebra;
use nalgebra as na;
pub use nalgebra::Matrix4;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
pub struct Context {
    backend: Backend,
}
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

        event_loop.run(move |event, _, control_flow| {
            let updated_screen_size: Option<na::Vector2<u32>> = None;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
            if let Some(size) = updated_screen_size {
                context
                    .backend
                    .resize_renderer(size)
                    .expect("failed to resize");
            }
            context
                .backend
                .begin_render()
                .expect("failed to start rendering frame");
            let mut child = ContextChild::new(&mut context);
            render.render_frame(&mut child);
            if child.quit {
                *control_flow = ControlFlow::Exit
            }

            context
                .backend
                .finish_render()
                .expect("failed to swap framebuffer");
        });
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
    pub fn build_meshes(&mut self, mesh: MeshAsset, texture: Texture) -> Mesh {
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
            texture: MeshTexture::RegularTexture(texture),
        }
    }
    pub fn build_texture(&mut self, image: &RgbaImage) -> Result<Texture> {
        self.context.backend.allocate_texture(image)
    }
    pub fn draw_mesh(&mut self, transform: Matrix4<f32>, mesh: &Mesh) -> Result<()> {
        self.context.backend.draw_mesh(transform, mesh)
    }
    pub fn build_framebuffer(&mut self, resolution: na::Vector2<u32>) -> Result<Framebuffer> {
        self.context.backend.build_framebuffer(resolution)
    }
    pub fn bind_framebuffer(&mut self, bound_framebuffer: &BoundFramebuffer) -> Result<()> {
        self.context.backend.bind_framebuffer(bound_framebuffer)
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
    fn render_frame<'a>(&mut self, context: &mut ContextChild<'a>);
}
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector2;

    struct EmptyRenderable {}
    impl Renderable for EmptyRenderable {
        fn init<'a>(_context: &mut ContextChild<'a>) -> Self {
            Self {}
        }
        fn render_frame<'a>(&mut self, context: &mut ContextChild<'a>) {
            context.quit();
        }
    }
    struct TriangleRenderable {
        num_frames: usize,
        triangle: Mesh,
        #[allow(dead_code)]
        texture: Texture,
    }
    impl Renderable for TriangleRenderable {
        fn init<'a>(context: &mut ContextChild<'a>) -> Self {
            let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 0, 0, 0]));
            let texture = context
                .build_texture(&image)
                .expect("failed to create image");
            let triangle = context.build_meshes(MeshAsset::new_triangle(), texture);
            Self {
                triangle,
                num_frames: 0,
                texture,
            }
        }
        fn render_frame<'a>(&mut self, context: &mut ContextChild<'a>) {
            if self.num_frames <= 10_000 {
                context
                    .draw_mesh(Matrix4::identity(), &self.triangle)
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
