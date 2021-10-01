use super::prelude::{
    Camera, EventCollector, GraphLayer, GraphNode, GraphType, GraphWeight, ModelRenderData, Ray,
    RenderLayer, Transform,
};
use asset_manager::AssetManager;
use legion::systems::CommandBuffer;
use legion::*;
use parry3d::{
    query::{Ray as PRay, RayCast},
    shape::HeightField as PHeightField,
};
use std::sync::Mutex;
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::{DMatrix, Isometry3, Point3, Vector2, Vector3},
    Context, DrawableTexture,
};
pub struct Grid<T> {
    data: Vec<T>,
    dimensions: Vector2<usize>,
}
impl<T> Grid<T> {
    pub fn from_fn<F: Fn(usize, usize) -> T>(f: F, dimensions: Vector2<usize>) -> Self {
        let mut data = vec![];
        data.reserve(dimensions.x * dimensions.y);
        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                data.push(f(x, y));
            }
        }

        Self { dimensions, data }
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
        assert!(x < self.dimensions.x);
        assert!(y < self.dimensions.y);
        &self.data[x * self.dimensions.y + y]
    }
    pub fn dimensions(&self) -> Vector2<usize> {
        self.dimensions
    }
    #[allow(dead_code)]
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        assert!(x < self.dimensions.x);
        assert!(y < self.dimensions.y);
        &mut self.data[x * self.dimensions.y + y]
    }
}
pub struct InsertableTerrain {}
pub struct Terrain {
    heights: DMatrix<f32>,
    collider: PHeightField,

    dimensions: Vector2<usize>,
}
impl Terrain {
    pub fn dimensions(&self) -> Vector2<usize> {
        self.dimensions
    }
    pub fn new_flat(dimensions: Vector2<usize>) -> Self {
        let heights = DMatrix::from_fn(dimensions.x, dimensions.y, |x, y| 0.0);
        let collider = PHeightField::new(heights.clone(), Vector3::new(1.0, 1.0, 1.0));
        Self {
            heights,
            dimensions,
            collider,
        }
    }
    pub fn new_cone(
        dimensions: Vector2<usize>,
        center: Vector2<f32>,
        slope: f32,
        center_height: f32,
    ) -> Self {
        let mut heights = vec![];
        heights.reserve(dimensions.x * dimensions.y);
        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                let radius = ((x as f32 - center.x).powi(2) + (y as f32 - center.y).powi(2)).sqrt();
                let height = center_height + radius * slope;
                heights.push(height);
            }
        }

        let heights = Grid::from_fn(
            move |x, y| {
                let radius = ((x as f32 - center.x).powi(2) + (y as f32 - center.y).powi(2)).sqrt();
                center_height + radius * slope
            },
            dimensions,
        );
        let heights = DMatrix::from_fn(dimensions.x, dimensions.y, move |x, y| {
            let radius = ((x as f32 - center.x).powi(2) + (y as f32 - center.y).powi(2)).sqrt();
            center_height + radius * slope
        });
        let collider = PHeightField::new(heights.clone(), Vector3::new(1.0, 1.0, 1.0));
        let isometry = Isometry3::translation(0.5, 0.0, 0.5);
        let aabb = collider.aabb(&isometry);
        println!("aabb{{\nmin:\n{}\nmax:\n{}\n}}", aabb.mins, aabb.maxs);
        println!("dimensions: {}", dimensions);
        Self {
            heights,
            dimensions,
            collider,
        }
    }
    pub fn insert(
        self,
        world: &mut World,
        resources: &mut Resources,
        model_manager: &mut AssetManager<sukakpak::Mesh>,
        context: &mut Context,
    ) -> Result<()> {
        let graph_layer: Box<dyn GraphLayer> = (&self).into();
        let (mesh, texture) = {
            let mut layers = resources.get_mut_or_insert::<Vec<Mutex<Box<dyn GraphLayer>>>>(vec![]);
            layers.push(Mutex::new(graph_layer));
            let texture = context.build_texture(&RgbaImage::from_pixel(
                100,
                100,
                Rgba::from([200, 200, 200, 200]),
            ))?;

            let mut vertices = vec![];
            for x in 0..self.dimensions.x - 1 {
                for y in 0..self.dimensions.y - 1 {
                    let x0_y0 = Vector3::new(x as f32, self.heights[(x, y)], y as f32);
                    let x0_y1 = Vector3::new(x as f32, self.heights[(x, y + 1)], y as f32 + 1.0);
                    let x1_y0 = Vector3::new(x as f32 + 1.0, self.heights[(x + 1, y)], y as f32);
                    let x1_y1 =
                        Vector3::new(x as f32 + 1.0, self.heights[(x + 1, y + 1)], y as f32 + 1.0);
                    let triangle0_normal = (x0_y1 - x0_y0).cross(&(x1_y0 - x0_y0)).normalize();
                    let triangle1_normal = (x1_y0 - x1_y1).cross(&(x0_y1 - x1_y1)).normalize();

                    //triangle 0
                    vertices.push([
                        //position:
                        x0_y0.x,
                        x0_y0.y,
                        x0_y0.z,
                        //uv
                        0.0,
                        0.0,
                        //normal
                        triangle0_normal.x,
                        triangle0_normal.y,
                        triangle0_normal.z,
                    ]);
                    vertices.push([
                        //position:
                        x0_y1.x,
                        x0_y1.y,
                        x0_y1.z,
                        //uv
                        0.0,
                        1.0,
                        //normal
                        triangle0_normal.x,
                        triangle0_normal.y,
                        triangle0_normal.z,
                    ]);
                    vertices.push([
                        //position:
                        x1_y0.x,
                        x1_y0.y,
                        x1_y0.z,
                        //uv
                        1.0,
                        0.0,
                        //normal
                        triangle0_normal.x,
                        triangle0_normal.y,
                        triangle0_normal.z,
                    ]);
                    //triangle 1
                    vertices.push([
                        //position:
                        x0_y1.x,
                        x0_y1.y,
                        x0_y1.z,
                        //uv
                        0.0,
                        1.0,
                        //normal
                        triangle1_normal.x,
                        triangle1_normal.y,
                        triangle1_normal.z,
                    ]);
                    vertices.push([
                        //position:
                        x1_y1.x,
                        x1_y1.y,
                        x1_y1.z,
                        //uv
                        1.0,
                        1.0,
                        //normal
                        triangle1_normal.x,
                        triangle1_normal.y,
                        triangle1_normal.z,
                    ]);
                    vertices.push([
                        //position:
                        x1_y0.x,
                        x1_y0.y,
                        x1_y0.z,
                        //uv
                        1.0,
                        0.0,
                        //normal
                        triangle1_normal.x,
                        triangle1_normal.y,
                        triangle1_normal.z,
                    ]);
                }
            }
            let indices = (0..self.dimensions.x * self.dimensions.y * 6)
                .map(|i| i as u32)
                .collect();

            let mesh = sukakpak::MeshAsset {
                indices,
                vertices: vertices
                    .iter()
                    .flatten()
                    .map(|f| f.to_ne_bytes())
                    .flatten()
                    .collect(),
                vertex_layout: sukakpak::VertexLayout {
                    components: vec![
                        sukakpak::VertexComponent::Vec3F32,
                        sukakpak::VertexComponent::Vec2F32,
                        sukakpak::VertexComponent::Vec3F32,
                    ],
                },
            };

            let model = context
                .build_mesh(mesh, DrawableTexture::Texture(&texture))
                .expect("failed to build mesh");
            (model, texture)
        };

        let model = model_manager.insert(mesh);

        world.push((
            InsertableTerrain {},
            Transform::default(),
            ModelRenderData::default(),
            model,
            texture,
        ));
        resources.insert(self);
        Ok(())
    }
    pub fn get_height(&self, x: usize, y: usize) -> f32 {
        self.heights[(y + 1, x + 1)]
    }

    /// Casts ray and returns (if it exists) point of intersection
    pub fn cast_ray(&self, ray: &Ray) -> Option<Point3<f32>> {
        let isometry = Isometry3::translation(0.5, 0.0, 0.5);
        let cast_ray = PRay {
            dir: Vector3::new(
                ray.direction.x / self.dimensions().x as f32,
                ray.direction.y,
                ray.direction.z / self.dimensions().y as f32,
            ),
            origin: Point3::new(
                ray.origin.x / self.dimensions().x as f32,
                ray.origin.y,
                ray.origin.z / self.dimensions().y as f32,
            ),
        };

        if let Some(dist) = self.collider.cast_ray(&isometry, &cast_ray, 10000.0, true) {
            let loc = cast_ray.point_at(dist);
            Some(Point3::new(
                loc.x * self.dimensions().x as f32,
                loc.y,
                loc.z * self.dimensions().y as f32,
            ))
        } else {
            None
        }
    }
}
struct TerrainWeight(f32);
pub struct TerrainGraphLayer {
    grid: Grid<TerrainWeight>,
}
pub struct Highlighted {}
#[system]
pub fn insert_highlited(
    command_buffer: &mut CommandBuffer,
    #[resource] graphics: &mut Context,
    #[resource] model_manager: &mut AssetManager<sukakpak::Mesh>,
) {
    println!("inserting highlighted");
    let texture = graphics
        .build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([250, 20, 20, 200]),
        ))
        .expect("failed to build lift");
    let model = model_manager.insert(
        graphics
            .build_mesh(
                sukakpak::MeshAsset::new_cube(),
                DrawableTexture::Texture(&texture),
            )
            .expect("failed to build lift mesh"),
    );
    command_buffer.push((
        ModelRenderData::default(),
        Highlighted {},
        texture,
        model,
        Transform::default().set_scale(Vector3::new(10.0, 10.0, 10.0)),
    ));
}
#[system(for_each)]
pub fn ray_cast(
    _highlighted: &Highlighted,
    transform: &mut Transform,
    #[resource] terrain: &Terrain,
    #[resource] events: &EventCollector,
    #[resource] camera: &mut Box<dyn Camera>,
) {
    let ray = camera.cast_mouse_ray(events.last_mouse_pos);
    if let Some(loc) = terrain.cast_ray(&ray) {
        *transform = transform
            .clone()
            .set_translation(Vector3::new(loc.x, loc.y, loc.z));
    };
}
impl From<&Terrain> for TerrainGraphLayer {
    fn from(t: &Terrain) -> Self {
        let mut data = vec![];
        data.reserve(t.heights.ncols() * t.heights.nrows());
        for x in 0..t.heights.ncols() {
            for y in 0..t.heights.nrows() {
                data.push(TerrainWeight(t.heights[(y, x)]));
            }
        }
        let dimensions = Vector2::new(t.heights.ncols(), t.heights.nrows());
        Self {
            grid: Grid { data, dimensions },
        }
    }
}
impl From<&Terrain> for Box<dyn GraphLayer> {
    fn from(t: &Terrain) -> Self {
        let layer: TerrainGraphLayer = t.into();
        Box::new(layer)
    }
}
impl GraphLayer for TerrainGraphLayer {
    fn get_type(&self) -> GraphType {
        GraphType::Terrain
    }
    fn get_children(&self, point: &GraphNode) -> Vec<(GraphNode, GraphWeight)> {
        let pos = point.0;
        let mut v = vec![];
        if pos.x > 0 {
            v.push((
                GraphNode(Vector2::new(pos.x - 1, pos.y)),
                GraphWeight::Some(1),
            ));
        }
        if pos.y > 0 {
            v.push((
                GraphNode(Vector2::new(pos.x, pos.y - 1)),
                GraphWeight::Some(1),
            ));
        }
        if pos.x < self.grid.dimensions().x - 1 {
            v.push((
                GraphNode(Vector2::new(pos.x + 1, pos.y)),
                GraphWeight::Some(1),
            ));
        }
        if pos.y < self.grid.dimensions().y - 1 {
            v.push((
                GraphNode(Vector2::new(pos.x, pos.y + 1)),
                GraphWeight::Some(1),
            ));
        }
        v
    }
    fn get_distance(&self, start_point: &GraphNode, end_point: &GraphNode) -> GraphWeight {
        todo!()
    }
}
