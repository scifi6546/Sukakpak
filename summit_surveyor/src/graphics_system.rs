use super::prelude::{
    AssetManager, Camera, GuiRuntimeModel, GuiTransform, Model, PushBuilder, RenderingCtx, Result,
    Terrain, Transform,
};
use legion::*;
use log::debug;

pub struct RuntimeModel {
    pub mesh: sukakpak::Mesh,
}
/// Used for printing debug info
pub struct RuntimeDebugMesh {
    mesh: sukakpak::Mesh,
}
pub struct GraphicsSettings {}
#[derive(Clone)]
pub struct RuntimeModelId {
    id: String,
}
impl RuntimeModelId {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}
impl RuntimeModel {
    pub fn new(model: &Model, context: &mut RenderingCtx) -> Self {
        let mut ctx_ref = context.0.borrow_mut();
        let texture = ctx_ref
            .build_texture(&model.texture.clone().into())
            .expect("failed to create texture");
        let mesh = ctx_ref.build_mesh(model.mesh.clone(), texture);
        Self { mesh }
    }
}

pub fn insert_terrain(
    terrain: Terrain,
    world: &mut World,
    context: &mut RenderingCtx,
    asset_manager: &mut AssetManager<RuntimeModel>,
) -> Result<()> {
    let model = terrain.model();
    let transform = model.transform.clone();
    asset_manager.get_or_create("game_terrain", RuntimeModel::new(&model, context));
    world.push((
        terrain.build_graph(),
        terrain,
        transform.clone(),
        RuntimeModelId {
            id: "game_terrain".to_string(),
        },
    ));
    Ok(())
}

#[system(for_each)]
pub fn render_object(
    transform: &Transform,
    model: &RuntimeModelId,
    push: &mut PushBuilder,
    #[resource] context: &RenderingCtx,
    #[resource] camera: &Camera,
    #[resource] asset_manager: &mut AssetManager<RuntimeModel>,
) {
    debug!("running render object");
    let model = asset_manager.get(&model.id).unwrap();
    push.set_view_matrix(camera.get_matrix(context.0.borrow_mut().get_screen_size()));
    push.set_model_matrix(transform.build().clone());
    context
        .0
        .borrow_mut()
        .draw_mesh(push.to_slice(), &model.mesh)
        .expect("failed to draw");
}
#[system(for_each)]
pub fn render_debug(
    transform: &Transform,
    model: &RuntimeDebugMesh,
    push: &mut PushBuilder,
    #[resource] context: &RenderingCtx,
    #[resource] camera: &Camera,
) {
    push.set_model_matrix(transform.build().clone());
    push.set_view_matrix(camera.get_matrix(context.0.borrow_mut().get_screen_size()));
    context
        .0
        .borrow_mut()
        .draw_mesh(push.to_slice(), &model.mesh)
        .expect("failed to draw");
}
#[system(for_each)]
pub fn render_gui(
    transform: &GuiTransform,
    model: &GuiRuntimeModel,
    push: &mut PushBuilder,
    #[resource] context: &RenderingCtx,
) {
    debug!("running render object");

    push.set_model_matrix(transform.transform.build().clone());
    context
        .0
        .borrow_mut()
        .draw_mesh(push.to_slice(), &model.model.mesh)
        .expect("failed to draw gui");
}
