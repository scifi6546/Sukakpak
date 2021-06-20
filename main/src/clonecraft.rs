use sukakpak::{ContextChild, Matrix4, Mesh, MeshAsset, Texture};
pub struct CloneCraft {
    triangle: Mesh,
    #[allow(dead_code)]
    texture: Texture,
}

impl sukakpak::Renderable for CloneCraft {
    fn init<'a>(context: &mut ContextChild<'a>) -> Self {
        let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 0, 0, 0]));
        let texture = context
            .build_texture(&image)
            .expect("failed to create image");
        let triangle = context.build_meshes(MeshAsset::new_triangle(), texture);
        Self { triangle, texture }
    }
    fn render_frame<'a>(&mut self, context: &mut ContextChild<'a>) {
        context
            .draw_mesh(Matrix4::identity(), &self.triangle)
            .expect("failed to draw triangle");
    }
}
