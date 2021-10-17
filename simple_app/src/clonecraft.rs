use std::time::Duration;
use sukakpak::{image, nalgebra, Context, ContextTrait, DrawableTexture, Event, MeshAsset};
pub struct CloneCraft {
    triangle: sukakpak::Mesh,
    framebuffer: sukakpak::Framebuffer,
    plane: sukakpak::Mesh,
    plane_2: sukakpak::Mesh,
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
        let framebuffer = context
            .build_framebuffer(nalgebra::Vector2::new(100, 100))
            .expect("failed to build");
        let plane = context
            .build_mesh(
                MeshAsset::new_plane(),
                DrawableTexture::Framebuffer(&framebuffer),
            )
            .expect("failed to build");
        let red_texture = context
            .build_texture(&image::ImageBuffer::from_pixel(
                100,
                100,
                image::Rgba([0, 0, 255, 0]),
            ))
            .expect("failed to build texture");
        let plane_2 = context
            .build_mesh(
                MeshAsset::new_plane(),
                DrawableTexture::Texture(&red_texture),
            )
            .expect("failed to build");

        Self {
            triangle,
            plane,
            plane_2,
            framebuffer,
        }
    }
    fn render_frame(&mut self, _events: &[Event], mut context: Context, _delta_time: Duration) {
        let mat: nalgebra::Matrix4<f32> = nalgebra::Matrix4::<f32>::identity();
        let data: Vec<u8> = mat
            .as_slice()
            .iter()
            .flat_map(|f| f.to_ne_bytes())
            .collect();
        context
            .bind_framebuffer(sukakpak::Bindable::UserFramebuffer(&self.framebuffer))
            .expect("failed to bind");
        context
            .draw_mesh(data.clone(), &self.triangle)
            .expect("failed to draw");
        context
            .bind_framebuffer(sukakpak::Bindable::ScreenFramebuffer)
            .expect("failed to bind");
        context
            .draw_mesh(data, &self.plane)
            .expect("failed to draw");
    }
}
