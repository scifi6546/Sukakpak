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
impl<'a> ContextChild<'a> {
    pub fn draw_mesh() {
        todo!()
    }
    pub fn bind_framebuffer() {
        todo!()
    }
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
