use super::prelude::{
    Camera, ContainerAlignment, EventCollector, EventListener, GraphLayer, GraphNode, GraphType,
    GraphWeight, GuiComponent, GuiSquare, GuiState, ModelRenderData, MouseButtonEvent, RenderLayer,
    Terrain, Transform, VerticalContainer, VerticalContainerStyle,
};
use asset_manager::AssetManager;
use legion::systems::CommandBuffer;
use legion::*;
use std::sync::Mutex;
use sukakpak::{
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3},
    Context, DrawableTexture,
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
    #[resource] graphics: &mut Context,
    #[resource] terrain: &Terrain,
    #[resource] model_manager: &mut AssetManager<sukakpak::Mesh>,
    #[resource] layers: &mut Vec<Mutex<Box<dyn GraphLayer>>>,
) {
    let texture = graphics
        .build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([20, 20, 20, 200]),
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
    let t1 = Transform::default().set_translation(Vector3::new(0.0, terrain.get_height(0, 0), 0.0));
    let t2 =
        Transform::default().set_translation(Vector3::new(10.0, terrain.get_height(10, 10), 10.0));
    command_buffer.push((ModelRenderData::default(), model.clone(), t1, Lift {}));
    command_buffer.push((ModelRenderData::default(), model, t2, Lift {}));
    layers.push(Mutex::new(Box::new(LiftLayer {
        start: GraphNode(Vector2::new(0, 0)),
        end: GraphNode(Vector2::new(10, 10)),
    })))
}
pub struct LiftBuilder {}
#[derive(Debug, Clone, PartialEq, Eq)]
enum LiftBuild {
    None,
    First,
    Second,
}
pub struct LiftBuilderState {
    lift: LiftBuild,
}
impl Default for LiftBuilderState {
    fn default() -> Self {
        LiftBuilderState {
            lift: LiftBuild::None,
        }
    }
}
/// Denotes bottom lift station
pub struct LiftBottom {}
/// Denotes top lift station
pub struct LiftTop {}
#[system]
pub fn lift_builder_gui(
    command_buffer: &mut CommandBuffer,
    #[resource] context: &mut Context,
    #[resource] gui_state: &mut GuiState,
    #[resource] model_manager: &mut AssetManager<sukakpak::Mesh>,
    #[resource] texture_manager: &mut AssetManager<sukakpak::Texture>,
) {
    let default_tex = texture_manager.insert(
        context
            .build_texture(&RgbaImage::from_pixel(
                100,
                100,
                Rgba::from([100, 100, 100, 255]),
            ))
            .expect("failed to build default texture"),
    );
    let hover_tex = texture_manager.insert(
        context
            .build_texture(&RgbaImage::from_pixel(
                100,
                100,
                Rgba::from([0, 80, 80, 255]),
            ))
            .expect("failed to build default texture"),
    );

    let click_tex = texture_manager.insert(
        context
            .build_texture(&RgbaImage::from_pixel(
                100,
                100,
                Rgba::from([0, 100, 80, 255]),
            ))
            .expect("failed to build default texture"),
    );
    let (g1, g2) = GuiComponent::make_tupal(Box::new(
        VerticalContainer::new(
            vec![Box::new(
                GuiSquare::new(
                    Transform::default().set_scale(Vector3::new(0.1, 0.1, 1.0)),
                    default_tex.clone(),
                    hover_tex.clone(),
                    click_tex.clone(),
                    model_manager,
                    texture_manager,
                    context,
                )
                .expect("failed to build square"),
            )],
            VerticalContainerStyle {
                alignment: ContainerAlignment::Center,
                padding: 0.01,
            },
            Vector3::new(0.0, 0.8, 0.5),
            context,
            gui_state,
            model_manager,
            texture_manager,
        )
        .expect("failed to make vert container"),
    ));
    command_buffer.push((g1, g2, LiftBuilder {}));
    let texture = context
        .build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([20, 0, 200, 200]),
        ))
        .expect("failed to build texture");
    let lift_model = model_manager.insert(
        context
            .build_mesh(
                sukakpak::MeshAsset::new_cube(),
                DrawableTexture::Texture(&texture),
            )
            .expect("failed to build mesh"),
    );
    command_buffer.push((
        lift_model.clone(),
        Transform::default().set_scale(Vector3::new(10.0, 10.0, 10.0)),
        LiftBottom {},
        ModelRenderData::default().with_new_layer(RenderLayer::DoNotRender),
    ));
    command_buffer.push((
        lift_model.clone(),
        Transform::default(),
        LiftTop {},
        ModelRenderData::default().with_new_layer(RenderLayer::DoNotRender),
    ));
}
#[system(for_each)]
pub fn bottom_lift(
    lift_bottom: &LiftBottom,
    model_render_data: &mut ModelRenderData,
    #[resource] builder_state: &LiftBuilderState,
) {
    if builder_state.lift == LiftBuild::First {
        model_render_data.set_render_layer(RenderLayer::Main);
        println!("setting render layer to main");
    } else {
        model_render_data.set_render_layer(RenderLayer::DoNotRender);
    }
}
#[system(for_each)]
pub fn run_lift_builder_gui(
    listener: &EventListener,
    lift_builder: &LiftBuilder,
    #[resource] builder_state: &mut LiftBuilderState,
    #[resource] terrain: &Terrain,
    #[resource] events: &EventCollector,
    #[resource] camera: &mut Box<dyn Camera>,
) {
    let clicked = match listener.sublistners[0].left_mouse_down {
        MouseButtonEvent::Clicked { .. } => true,
        _ => false,
    };
    if clicked {
        let new_state = match builder_state.lift {
            LiftBuild::None => LiftBuild::First,
            LiftBuild::First => LiftBuild::Second,
            LiftBuild::Second => LiftBuild::None,
        };
        builder_state.lift = new_state;
    }
}
