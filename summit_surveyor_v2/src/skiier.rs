use super::prelude::{
    dijkstra, GraphLayer, GraphNode, Model, Path, RenderingCtx, Terrain, Transform,
};
use legion::systems::CommandBuffer;
use legion::*;
use std::sync::Mutex;
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3},
};
pub struct FollowPath {
    points: Vec<Vector3<f32>>,
    t: f32,
}
impl FollowPath {
    pub fn incr(&mut self, delta_t: f32) -> Vector3<f32> {
        self.t += delta_t;
        let mix = self.t - self.t.floor();
        if (self.t.ceil() as usize) < self.points.len() {
            self.points[self.t.floor() as usize] * (1.0 - mix)
                + self.points[self.t.ceil() as usize] * mix
        } else if (self.t.floor() as usize) < self.points.len() {
            self.points[self.t.floor() as usize]
        } else {
            *self.points.last().unwrap()
        }
    }
}
pub struct Skiier {}
impl Skiier {
    pub fn insert(
        start: Vector2<usize>,
        end: Vector2<usize>,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<()> {
        let layers: &Vec<Mutex<Box<dyn GraphLayer>>> = &resources.get().unwrap();
        let path = dijkstra(&GraphNode(start), &GraphNode(end), layers.as_slice());
        let terrain: &Terrain = &resources.get().unwrap();

        let follow = FollowPath {
            points: path
                .path
                .iter()
                .map(|p| {
                    let x = p.0 .0.x;
                    let z = p.0 .0.y;
                    let y = terrain.get(x, z);
                    Vector3::new(x as f32, y as f32, z as f32)
                })
                .collect(),
            t: 0.0,
        };
        let transform = Transform::default().set_translation(follow.points[0]);
        let mut r_ctx: &RenderingCtx = &resources.get_mut().unwrap();
        let texture = r_ctx.0.borrow_mut().build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([20, 200, 200, 200]),
        ))?;
        let model = Model::new(
            r_ctx
                .0
                .borrow_mut()
                .build_mesh(sukakpak::MeshAsset::new_cube(), texture),
        );
        println!("path: {}", path);
        world.push((Skiier {}, follow, transform, model));
        Ok(())
    }
}
#[system(for_each)]
pub fn skiier(path: &mut FollowPath, transform: &mut Transform) {
    *transform = transform.clone().set_translation(path.incr(0.01));
}
