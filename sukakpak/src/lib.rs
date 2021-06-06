use generational_arena::{Arena, Index as ArenaIndex};
pub struct Context {}
impl Context {
    pub fn new<R: Renderable>() -> ! {
        let mut context = Context {};
        let mut render = {
            let mut child = ContextChild {
                context: &mut context,
            };
            R::init(&mut child)
        };
        loop {
            let mut child = ContextChild {
                context: &mut context,
            };
            render.render_frame(&mut child);
        }
    }
}
pub struct ContextChild<'a> {
    context: &'a mut Context,
}
pub struct Mesh {}
pub struct Texture {}
pub struct FrameBuffer {}
//draws meshes. Will draw on update_uniform, bind_framebuffer, or force_draw
impl<'a> ContextChild<'a> {
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
        todo!()
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
        Context::new::<EmptyRenderable>();
    }
}
