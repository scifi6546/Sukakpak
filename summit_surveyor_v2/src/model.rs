use super::prelude::{Camera, Transform};
use asset_manager::{AssetHandle, AssetManager};
use legion::*;
use std::sync::Mutex;
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::Vector2,
    Bindable, Context, ContextTrait, DrawableTexture, VertexComponent, VertexLayout,
};
pub struct ScreenPlane {
    pub framebuffer: sukakpak::Framebuffer,
    pub mesh: sukakpak::Mesh,
}
pub fn build_screen_plane(
    context: &mut Context,
    shader_name: &str,
    screen_resolution: Vector2<u32>,
    z: f32,
) -> Result<ScreenPlane> {
    let framebuffer = context.build_framebuffer(screen_resolution)?;
    context.bind_shader(Bindable::UserFramebuffer(&framebuffer), shader_name);

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

#[system(for_each)]
pub fn render_model_vec(
    mesh_vec: &Vec<(AssetHandle<sukakpak::Mesh>, Transform)>,
    render_data: &ModelRenderData,
    #[resource] camera: &mut Box<dyn Camera>,
    #[resource] manager: &AssetManager<sukakpak::Mesh>,
    #[resource] graphics: &mut Context,
) {
    if render_data.get_render_layer() == RenderLayer::Main {
        for (model, transform) in mesh_vec.iter() {
            graphics
                .draw_mesh(
                    camera.to_vec(transform),
                    &manager.get(model).expect("model does not exist"),
                )
                .expect("failed to draw mesh");
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderLayer {
    /// Never render the object
    DoNotRender,
    /// Render in main world
    Main,
}
pub struct ModelRenderData {
    layer: RenderLayer,
}
impl Default for ModelRenderData {
    fn default() -> Self {
        Self {
            layer: RenderLayer::Main,
        }
    }
}
impl ModelRenderData {
    pub fn get_render_layer(&self) -> RenderLayer {
        self.layer
    }
    pub fn with_new_layer(self, layer: RenderLayer) -> Self {
        Self { layer }
    }
    pub fn set_render_layer(&mut self, layer: RenderLayer) {
        self.layer = layer
    }
}

#[system(for_each)]
pub fn render_model(
    model: &AssetHandle<sukakpak::Mesh>,
    render_data: &ModelRenderData,
    transform: &Transform,
    #[resource] camera: &mut Box<dyn Camera>,
    #[resource] manager: &AssetManager<sukakpak::Mesh>,
    #[resource] graphics: &mut Context,
) {
    if render_data.get_render_layer() == RenderLayer::Main {
        graphics
            .draw_mesh(
                camera.to_vec(transform),
                &manager.get(model).expect("model does not exist"),
            )
            .expect("failed to draw mesh");
    }
}
