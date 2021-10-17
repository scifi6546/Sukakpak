use std::time::Duration;
use sukakpak::{image, nalgebra, Context, ContextTrait, DrawableTexture, Event, MeshAsset};
pub struct CloneCraft {
    triangle: sukakpak::Mesh,
}
impl sukakpak::Renderable for CloneCraft {
    fn init(mut context: Context) -> Self {
        let blue_texture = context
            .build_texture(&image::ImageBuffer::from_pixel(
                100,
                100,
                image::Rgba([0, 0, 255, 0]),
            ))
            .expect("failed to build texture");

        let triangle = context
            .build_mesh(
                MeshAsset::new_cube(),
                DrawableTexture::Texture(&blue_texture),
            )
            .expect("failed to build mesh");
        Self { triangle }
    }
    fn render_frame(&mut self, _events: &[Event], mut context: Context, _delta_time: Duration) {
        let mat: nalgebra::Matrix4<f32> = nalgebra::Matrix4::<f32>::identity();
        let data = mat
            .as_slice()
            .iter()
            .flat_map(|f| f.to_ne_bytes())
            .collect();
        context
            .draw_mesh(data, &self.triangle)
            .expect("failed to draw");
    }
}
