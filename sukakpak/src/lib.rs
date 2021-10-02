pub use anyhow;
pub use image;
pub use nalgebra;
mod mesh;
pub use mesh::{EasyMesh, Mesh as MeshAsset, Vertex as EasyMeshVertex};
mod vulkan;
pub use vulkan::{
    events::{Event, MouseButton},
    Bindable, Context, CreateInfo, DrawableTexture, Framebuffer, Mesh, MeshTexture, Renderable,
    Sukakpak, Texture, VertexComponent, VertexLayout,
};
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Matrix4, Vector2};
    use std::time::Duration;
    struct EmptyRenderable {}
    impl Renderable for EmptyRenderable {
        fn init<'a>(_context: Context) -> Self {
            Self {}
        }
        fn render_frame<'a>(
            &mut self,
            _events: &[Event],
            mut context: Context,
            _delta_time: Duration,
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
        fn init<'a>(mut context: Context) -> Self {
            let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 0, 0, 0]));
            let texture = context
                .build_texture(&image)
                .expect("failed to create image");
            let triangle = context
                .build_mesh(MeshAsset::new_triangle(), texture)
                .unwrap();
            Self {
                triangle,
                num_frames: 0,
                texture,
            }
        }
        fn render_frame<'a>(&mut self, _events: &[Event], mut context: Context, _dt: Duration) {
            if self.num_frames <= 10_000 {
                let mat = Matrix4::<f32>::identity();
                context
                    .draw_mesh(
                        mat.as_slice()
                            .iter()
                            .map(|f| f.to_ne_bytes())
                            .flatten()
                            .collect(),
                        &self.triangle,
                    )
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
        Sukakpak::new::<TriangleRenderable>(CreateInfo {
            default_size: Vector2::new(800, 800),
            name: String::from("Draw Triangle"),
        });
    }
}
