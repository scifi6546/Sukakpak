use generational_arena::{Arena, Index as ArenaIndex};
mod backend;
use backend::Backend;
pub use backend::BackendCreateInfo as CreateInfo;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
pub struct Context {
    backend: Backend,
}
impl Context {
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
            let mut child = ContextChild::new(&mut context);
            render.render_frame(&mut child);
            if child.quit {
                *control_flow = ControlFlow::Exit
            }
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }
}
pub struct ContextChild<'a> {
    context: &'a mut Context,
    //true if quit is signaled
    quit: bool,
}
pub struct Mesh {}
pub struct Texture {}
pub struct FrameBuffer {}
//draws meshes. Will draw on update_uniform, bind_framebuffer, or force_draw
impl<'a> ContextChild<'a> {
    fn new(context: &'a mut Context) -> Self {
        Self {
            context,
            quit: false,
        }
    }
    pub fn build_meshes(&mut self) -> Mesh {
        todo!("build mesh")
    }
    pub fn build_texture(&mut self) -> Texture {
        todo!("build texture")
    }
    pub fn draw_mesh(&mut self, mesh: Mesh) {
        todo!("draw mesh")
    }
    pub fn build_framebuffer(&mut self) {
        todo!("build framebuffer")
    }
    pub fn bind_framebuffer(&mut self) {
        todo!("bind framebuffer")
    }
    pub fn update_uniform(&mut self) {
        todo!("update uniform")
    }
    pub fn force_draw(mut self) {
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
    #[test]
    fn it_works() {
        //should start and stop without issue
        Context::new::<EmptyRenderable>(CreateInfo {
            default_size: Vector2::new(800, 800),
            name: String::from("Basic Unit Test"),
        });
    }
}