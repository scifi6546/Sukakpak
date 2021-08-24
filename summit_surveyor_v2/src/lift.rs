use super::prelude::{
    GraphLayer, GraphNode, GraphType, GraphWeight, Model, RenderingCtx, Terrain, Transform,
};
use legion::systems::CommandBuffer;
use legion::*;
use std::sync::Mutex;
use sukakpak::{
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3},
};
pub struct Lift {}
pub struct LiftLayer {
    start: GraphNode,
    end: GraphNode,
}
impl GraphLayer for LiftLayer {
    fn get_type(&self) -> GraphType {
        GraphType::Lift {
            start: self.start,
            end: self.end,
        }
    }
    fn get_children(&self, point: &GraphNode) -> Vec<(GraphNode, GraphWeight)> {
        if *point == self.start {
            vec![(self.end, GraphWeight::Some(1))]
        } else {
            vec![]
        }
    }

    fn get_distance(&self, start_point: &GraphNode, end_point: &GraphNode) -> GraphWeight {
        if *start_point == self.start && *end_point == self.end {
            GraphWeight::Some(1)
        } else {
            GraphWeight::Infinity
        }
    }
}
#[system]
pub fn insert_lift(
    command_buffer: &mut CommandBuffer,
    #[resource] graphics: &mut RenderingCtx,
    #[resource] terrain: &Terrain,
    #[resource] layers: &mut Vec<Mutex<Box<dyn GraphLayer>>>,
) {
    let texture = graphics
        .0
        .borrow_mut()
        .build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([20, 20, 20, 200]),
        ))
        .expect("failed to build lift");
    let model = Model::new(
        graphics
            .0
            .borrow_mut()
            .build_mesh(sukakpak::MeshAsset::new_cube(), texture),
    );
    let t1 = Transform::default().set_translation(Vector3::new(0.0, terrain.get_height(0, 0), 0.0));
    let t2 =
        Transform::default().set_translation(Vector3::new(10.0, terrain.get_height(10, 10), 10.0));
    command_buffer.push((model.clone(), t1, Lift {}));
    command_buffer.push((model, t2, Lift {}));
    layers.push(Mutex::new(Box::new(LiftLayer {
        start: GraphNode(Vector2::new(0, 0)),
        end: GraphNode(Vector2::new(10, 10)),
    })))
}