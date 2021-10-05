use super::prelude::{
    dijkstra, GraphLayer, GraphNode, GraphType, GraphWeight, ModelRenderData, Path, Terrain,
    Transform,
};
use asset_manager::AssetManager;
mod decision_tree;
use decision_tree::{DecisionCost, DecisionTree};
use legion::*;
use std::{sync::Mutex, time::Duration};
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3},
    Context, ContextTrait, DrawableTexture, Texture,
};
pub struct FollowPath {
    start: GraphNode,
    end: GraphNode,
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
    pub fn at_end(&self) -> bool {
        self.t >= self.points.len() as f32
    }
}
pub struct Skiier {}
impl Skiier {
    pub fn insert(
        start: Vector2<usize>,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<()> {
        let layers: &Vec<Mutex<Box<dyn GraphLayer>>> = &resources.get().unwrap();
        let terrain: &Terrain = &resources.get().unwrap();
        let model_manager: &mut AssetManager<sukakpak::Mesh> = &mut resources.get_mut().unwrap();
        let texture_manager: &mut AssetManager<Texture> = &mut resources.get_mut().unwrap();

        let (decison_tree, cost, path) = DecisionTree::new(GraphNode(start), layers);
        let follow = FollowPath {
            start: path.get(0).unwrap().0,
            end: *path.endpoint().unwrap(),
            points: path
                .path
                .iter()
                .map(|p| {
                    let x = p.0 .0.x;
                    let z = p.0 .0.y;
                    let y = terrain.get_height(x, z);
                    Vector3::new(x as f32, y as f32, z as f32)
                })
                .collect(),
            t: 0.0,
        };
        let transform = Transform::default().set_translation(follow.points[0]);
        let r_ctx: &mut Context = &mut resources.get_mut().unwrap();

        let texture = r_ctx.build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([20, 200, 200, 200]),
        ))?;
        let model = model_manager.insert(
            r_ctx
                .build_mesh(
                    sukakpak::MeshAsset::new_cube(),
                    DrawableTexture::Texture(&texture),
                )
                .expect("failed to build mesh"),
        );
        texture_manager.insert(texture);
        println!("path: {}", path);
        world.push((
            ModelRenderData::default(),
            Skiier {},
            follow,
            transform,
            model,
            decison_tree,
            cost,
        ));
        Ok(())
    }
}
/// Recalculates path once at end of following it
#[system(for_each)]
pub fn skiier_path(
    follow_path: &mut FollowPath,
    decison_tree: &mut DecisionTree,
    cost: &mut DecisionCost,
    #[resource] layers: &Vec<Mutex<Box<dyn GraphLayer>>>,
    #[resource] terrain: &Terrain,
) {
    if follow_path.at_end() {
        let (new_decison_tree, new_cost, new_path) = DecisionTree::new(follow_path.end, layers);
        *follow_path = FollowPath {
            start: follow_path.start,
            end: follow_path.end,
            points: new_path
                .path
                .iter()
                .map(|p| {
                    let x = p.0 .0.x;
                    let z = p.0 .0.y;
                    let y = terrain.get_height(x, z);
                    Vector3::new(x as f32, y as f32, z as f32)
                })
                .collect(),
            t: 0.0,
        };
        *cost = new_cost;
        *decison_tree = new_decison_tree;
    }
}

#[system(for_each)]
pub fn skiier(
    path: &mut FollowPath,
    transform: &mut Transform,
    #[resource] duration: &mut Duration,
) {
    *transform = transform
        .clone()
        .set_translation(path.incr(1000.0 * duration.as_secs_f32()));
}
