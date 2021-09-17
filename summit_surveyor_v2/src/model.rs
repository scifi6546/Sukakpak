use super::prelude::{Camera, Transform};
use asset_manager::{AssetHandle, AssetManager};
use legion::*;
use std::sync::Mutex;
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::Vector2,
    Bindable, Context, DrawableTexture, VertexComponent, VertexLayout,
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
pub fn insert_cube(
    transform: Transform,
    world: &mut World,
    manager: &mut AssetManager<sukakpak::Mesh>,
    context: &mut Context,
) -> Result<()> {
    let vertices: Vec<((f32, f32, f32), (f32, f32), (f32, f32, f32))> = vec![
        (
            //position
            (-1.0, -1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (1.0, 1.0, 1.0),
            //uv
            (1.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (1.0, -1.0, 1.0),
            //uv
            (1.0, 0.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        //second triangle
        (
            //position
            (-1.0, -1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (-1.0, 1.0, 1.0),
            //uv
            (0.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (1.0, 1.0, 1.0),
            //uv
            (1.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        //third triangle
        (
            //position
            (1.0, -1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (1.0, 0.0, 0.0),
        ),
        (
            //position
            (1.0, 1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (1.0, -1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        //fourth triangle
        (
            //position
            (1.0, -1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (1.0, 1.0, 1.0),
            //uv
            (0.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (1.0, 1.0, -1.0),
            //uv
            (1.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        //fith triangle
        (
            //position
            (1.0, -1.0, -1.0),
            //uv
            (0.0, 0.0),
            //normal
            (0.0, 0.0, -1.0),
        ),
        (
            //position
            (-1.0, -1.0, -1.0),
            //uv
            (1.0, 0.0),
            //normal
            (0.0, 0.0, -1.0),
        ),
        (
            //position
            (1.0, 1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (0.0, 0.0, -1.0),
        ),
        //sixth triangle
        (
            //position
            (-1.0, -1.0, -1.0),
            //uv
            (1.0, 0.0),
            //normal
            (0.0, 0.0, -1.0),
        ),
        (
            //position
            (-1.0, 1.0, -1.0),
            //uv
            (1.0, 1.0),
            //normal
            (0.0, 0.0, -1.0),
        ),
        (
            //position
            (1.0, 1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (0.0, 0.0, -1.0),
        ),
        //seventh triangle
        (
            //position
            (-1.0, -1.0, -1.0),
            //uv
            (0.0, 0.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (-1.0, -1.0, 1.0),
            //uv
            (1.0, 0.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (-1.0, 1.0, 1.0),
            //uv
            (1.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        //eighth triangle
        (
            //position
            (-1.0, -1.0, -1.0),
            //uv
            (0.0, 0.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (-1.0, 1.0, 1.0),
            //uv
            (1.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        (
            //position
            (-1.0, 1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (-1.0, 0.0, 0.0),
        ),
        //9th triangle
        (
            //position
            (1.0, 1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (0.0, 1.0, 0.0),
        ),
        (
            //position
            (1.0, 1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (0.0, 1.0, 0.0),
        ),
        (
            //position
            (-1.0, 1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (0.0, 1.0, 0.0),
        ),
        //10th triangle
        (
            //position
            (1.0, 1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (0.0, 1.0, 0.0),
        ),
        (
            //position
            (-1.0, 1.0, -1.0),
            //uv
            (1.0, 1.0),
            //normal
            (0.0, 1.0, 0.0),
        ),
        (
            //position
            (-1.0, 1.0, 1.0),
            //uv
            (0.0, 1.0),
            //normal
            (0.0, 1.0, 0.0),
        ),
        //11th triangle
        (
            //position
            (1.0, -1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (0.0, -1.0, 0.0),
        ),
        (
            //position
            (-1.0, -1.0, 1.0),
            //uv
            (1.0, 0.0),
            //normal
            (0.0, -1.0, 0.0),
        ),
        (
            //position
            (-1.0, -1.0, -1.0),
            //uv
            (1.0, 1.0),
            //normal
            (0.0, -1.0, 0.0),
        ),
        //12th triangle
        (
            //position
            (1.0, -1.0, 1.0),
            //uv
            (0.0, 0.0),
            //normal
            (0.0, -1.0, 0.0),
        ),
        (
            //position
            (-1.0, -1.0, -1.0),
            //uv
            (1.0, 1.0),
            //normal
            (0.0, -1.0, 0.0),
        ),
        (
            //position
            (1.0, -1.0, -1.0),
            //uv
            (0.0, 1.0),
            //normal
            (0.0, -1.0, 0.0),
        ),
    ];

    let indices = vertices.iter().enumerate().map(|(i, _)| i as u32).collect();
    let texture = context.build_texture(&RgbaImage::from_pixel(
        100,
        100,
        Rgba::from([20, 200, 200, 200]),
    ))?;
    let vertices = vertices
        .iter()
        .map(|((pos_x, pos_y, pos_z), (uv_x, uv_y), (n_x, n_y, n_z))| {
            [pos_x, pos_y, pos_z, uv_x, uv_y, n_x, n_y, n_z]
        })
        .flatten()
        .map(|f| f.to_ne_bytes())
        .flatten()
        .collect();
    let mesh = context
        .build_mesh(
            sukakpak::MeshAsset {
                vertices,
                indices,
                vertex_layout: VertexLayout {
                    components: vec![
                        VertexComponent::Vec3F32,
                        VertexComponent::Vec2F32,
                        VertexComponent::Vec3F32,
                    ],
                },
            },
            DrawableTexture::Texture(&texture),
        )
        .expect("failed to build mesh");
    let handle = manager.insert(mesh);
    world.push((transform, handle, texture));
    Ok(())
}

#[system(for_each)]
pub fn render_model_vec(
    mesh_vec: &Vec<(AssetHandle<sukakpak::Mesh>, Transform)>,
    #[resource] camera: &mut Box<dyn Camera>,
    #[resource] manager: &AssetManager<sukakpak::Mesh>,
    #[resource] graphics: &mut Context,
) {
    for (model, transform) in mesh_vec.iter() {
        graphics
            .draw_mesh(
                camera.to_vec(transform),
                &manager.get(model).expect("model does not exist"),
            )
            .expect("failed to draw mesh");
    }
}
#[system(for_each)]
pub fn render_model(
    model: &AssetHandle<sukakpak::Mesh>,
    transform: &Transform,
    #[resource] camera: &mut Box<dyn Camera>,
    #[resource] manager: &AssetManager<sukakpak::Mesh>,
    #[resource] graphics: &mut Context,
) {
    graphics
        .draw_mesh(
            camera.to_vec(transform),
            &manager.get(model).expect("model does not exist"),
        )
        .expect("failed to draw mesh");
}
