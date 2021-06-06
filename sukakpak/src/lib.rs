use generational_arena::{Arena, Index as ArenaIndex};
pub struct Context {}
impl Context {
    pub fn new<R: Renderable>(mut render: R) -> ! {
        let mut context = Context {};
        loop {
            let child = ContextChild {
                context: &mut context,
            };
            render.render_frame(child);
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
    pub fn build_meshes() {}
    pub fn build_texture() {}
    pub fn draw_mesh() {
        todo!()
    }
    pub fn build_framebuffer() {}
    pub fn bind_framebuffer() {
        todo!()
    }
    pub fn update_uniform() {}
    pub fn force_draw() {}
}
/// User Provided code that provides draw calls
pub trait Renderable {
    fn render_frame<'a>(&mut self, context: ContextChild<'a>);
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
