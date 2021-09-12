use super::prelude::{Camera, Transform};
use asset_manager::{AssetHandle, AssetManager};
use legion::*;
use std::sync::Mutex;
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::Vector2,
    Context, DrawableTexture,
};
pub struct ScreenPlane {
    pub framebuffer: sukakpak::Framebuffer,
    pub mesh: sukakpak::Mesh,
}
pub fn build_screen_plane(
    context: &mut Context,
    screen_resolution: Vector2<u32>,
    z: f32,
) -> Result<ScreenPlane> {
    let framebuffer = context.build_framebuffer(screen_resolution)?;
    let vertices = [
        ((-1.0, -1.0, z), (0.0, 1.0)),
        ((1.0, -1.0, z), (1.0, 1.0)),
        ((-1.0, 1.0, z), (0.0, 0.0)),
        ((1.0, 1.0, z), (1.0, 0.0)),
    ]
    .iter()
    .map(|((x, y, z), (u, v))| [x, y, z, u, v])
    .flatten()
    .map(|f| f.to_ne_bytes())
    .flatten()
    .collect();
    let indices = vec![0, 1, 3, 0, 3, 2];
    let mesh = context.build_mesh(
        sukakpak::MeshAsset {
            indices,
            vertices,
            vertex_layout: sukakpak::VertexLayout {
                components: vec![
                    sukakpak::VertexComponent::Vec3F32,
                    sukakpak::VertexComponent::Vec2F32,
                ],
            },
        },
        sukakpak::DrawableTexture::Framebuffer(&framebuffer),
    )?;
    Ok(ScreenPlane { mesh, framebuffer })
}
#[derive(Debug)]
pub struct Model {
    pub mesh: sukakpak::Mesh,
}
impl Model {
    pub fn new(mesh: sukakpak::Mesh) -> Self {
        Self { mesh }
    }
}
pub fn insert_cube(
    transform: Transform,
    world: &mut World,
    manager: &mut AssetManager<Model>,
    context: &mut Context,
) -> Result<()> {
    let texture = context.build_texture(&RgbaImage::from_pixel(
        100,
        100,
        Rgba::from([20, 200, 200, 200]),
    ))?;
    let model = Model {
        mesh: context
            .build_mesh(
                sukakpak::MeshAsset::new_cube(),
                DrawableTexture::Texture(&texture),
            )
            .expect("failed to build mesh"),
    };
    let handle = manager.insert(model);
    world.push((transform, handle, texture));
    Ok(())
}

#[system(for_each)]
pub fn render_model(
    model: &AssetHandle<Model>,
    transform: &Transform,
    #[resource] camera: &Mutex<Box<dyn Camera>>,
    #[resource] manager: &AssetManager<Model>,
    #[resource] graphics: &mut Context,
) {
    graphics
        .draw_mesh(
            camera.lock().unwrap().to_vec(transform),
            &manager.get(model).expect("model does not exist").mesh,
        )
        .expect("failed to draw mesh");
}
