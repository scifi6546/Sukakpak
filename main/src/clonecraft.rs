pub struct CloneCraft {}
impl sukakpak::Renderable for CloneCraft {
    fn render_frame<'a>(&mut self, context: &mut sukakpak::ContextChild<'a>) {
        todo!()
    }
    fn init<'a>(context: &mut sukakpak::ContextChild<'a>) -> Self {
        let cube = context.build_meshes();
        todo!()
    }
}
