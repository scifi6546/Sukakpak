use super::prelude::{Camera, Transform};
use legion::*;
use std::{cell::RefCell, rc::Rc};
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::Vector2,
    Context,
};

pub struct Terrain {
    heights: Vec<f32>,
    dimensions: Vector2<usize>,
}
impl Terrain {
    pub fn new_flat(dimensions: Vector2<usize>) -> Self {
        let heights = vec![0.0; dimensions.x * dimensions.y];
        Self {
            heights,
            dimensions,
        }
    }
    pub fn insert(&self, world: &mut World, context: &Rc<RefCell<Context>>) -> Result<()> {
        let texture = context.borrow_mut().build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([200, 200, 200, 200]),
        ))?;
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut idx = 0;
        for x in 0..self.dimensions.x - 1 {
            for y in 0..self.dimensions.y - 1 {
                //Vertex 0:
                //position
                vertices.push(x as f32);
                vertices.push(y as f32);
                vertices.push(self.get(x, y));

                //texture coord
                vertices.push(0.0);
                vertices.push(0.0);
                //normal
                vertices.push(0.0);
                vertices.push(1.0);
                vertices.push(0.0);

                //Vertex 1

                //position
                vertices.push((x + 1) as f32);
                vertices.push(y as f32);
                vertices.push(self.get(x + 1, y));

                //texture coord
                vertices.push(1.0);
                vertices.push(0.0);
                //normal
                vertices.push(0.0);
                vertices.push(1.0);
                vertices.push(0.0);
                //Vertex2

                //position
                vertices.push((x) as f32);
                vertices.push((y + 1) as f32);
                vertices.push(self.get(x, y + 1));

                //texture coord
                vertices.push(0.0);
                vertices.push(1.0);
                //normal
                vertices.push(0.0);
                vertices.push(1.0);
                vertices.push(0.0);
                //Vertex3

                //position
                vertices.push((x + 1) as f32);
                vertices.push((y + 1) as f32);
                vertices.push(self.get(x + 1, y + 1));

                //texture coord
                vertices.push(1.0);
                vertices.push(1.0);
                //normal
                vertices.push(0.0);
                vertices.push(1.0);
                vertices.push(0.0);
                indices.push(idx);
                indices.push(idx + 2);
                indices.push(idx + 1);

                indices.push(idx + 2);
                indices.push(idx + 3);
                indices.push(idx + 1);
                idx += 4;
            }
        }
        let mesh = sukakpak::MeshAsset {
            indices,
            vertices: vertices.iter().map(|f| f.to_ne_bytes()).flatten().collect(),
            vertex_layout: sukakpak::VertexLayout {
                components: vec![
                    sukakpak::VertexComponent::Vec3F32,
                    sukakpak::VertexComponent::Vec2F32,
                    sukakpak::VertexComponent::Vec3F32,
                ],
            },
        };
        let model = Model {
            mesh: context.borrow_mut().build_mesh(mesh, texture),
        };
        world.push((Transform::default(), model, texture));
        Ok(())
    }
    fn get(&self, x: usize, y: usize) -> f32 {
        self.heights[x * self.dimensions.y + y]
    }
}
pub struct ScreenPlane {
    pub framebuffer: sukakpak::Framebuffer,
    pub mesh: sukakpak::Mesh,
}
pub fn build_screen_plane(
    context: Rc<RefCell<Context>>,
    screen_resolution: Vector2<u32>,
    z: f32,
) -> Result<ScreenPlane> {
    let framebuffer = context.borrow_mut().build_framebuffer(screen_resolution)?;
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
    let mesh = context.borrow_mut().build_mesh(
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
        sukakpak::MeshTexture::Framebuffer(framebuffer),
    );
    Ok(ScreenPlane { mesh, framebuffer })
}
#[derive(Debug, Clone)]
pub struct Model {
    mesh: sukakpak::Mesh,
}
pub fn insert_cube(
    transform: Transform,
    world: &mut World,
    context: Rc<RefCell<Context>>,
) -> Result<()> {
    let texture = context.borrow_mut().build_texture(&RgbaImage::from_pixel(
        100,
        100,
        Rgba::from([20, 200, 200, 200]),
    ))?;
    let model = Model {
        mesh: context
            .borrow_mut()
            .build_mesh(sukakpak::MeshAsset::new_cube(), texture),
    };
    world.push((transform, model, texture));
    Ok(())
}

unsafe impl Send for RenderingCtx {}
pub struct RenderingCtx(pub Rc<RefCell<Context>>);
impl RenderingCtx {
    pub fn new(ctx: &Rc<RefCell<Context>>) -> Self {
        Self(ctx.clone())
    }
}
#[system(for_each)]
pub fn render_model(
    model: &Model,
    transform: &Transform,
    #[resource] camera: &Camera,
    #[resource] graphics: &mut RenderingCtx,
) {
    graphics
        .0
        .borrow_mut()
        .draw_mesh(camera.to_vec(transform), &model.mesh)
        .expect("failed to draw mesh");
}