use super::prelude::{
    AssetManager, Camera, ContextChild, GuiRuntimeModel, GuiTransform, Model, PushBuilder, Result,
    Shader, ShaderBind, Terrain, Transform,
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
    pub fn new(model: &Model, context: &mut ContextChild, bound_shader: &Shader) -> Self {
        let texture = context
            .build_texture(&model.texture.into())
            .expect("failed to create texture");
        let mesh = context.build_mesh(model.mesh.clone(), texture);
        Self { mesh }
    }
}
impl RuntimeDebugMesh {
    pub fn new(model: &Model, context: &mut ContextChild, bound_shader: &Shader) -> Self {
        let texture = context
            .build_texture(&model.texture.into())
            .expect("failed to build texture");
        let mesh = context.build_mesh(model.mesh, texture);
        Self { mesh }
    }
}
pub fn insert_terrain(
    terrain: Terrain,
    world: &mut World,
    context: &mut ContextChild,
    asset_manager: &mut AssetManager<RuntimeModel>,
    bound_shader: &Shader,
) -> Result<()> {
    let model = terrain.model();
    let transform = model.transform.clone();
    asset_manager.get_or_create(
        "game_terrain",
        RuntimeModel::new(&model, context, bound_shader),
    );
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
    push: &PushBuilder,
    #[resource] settings: &GraphicsSettings,
    #[resource] context: &mut ContextChild,
    #[resource] shader: &ShaderBind,
    #[resource] camera: &Camera,
    #[resource] asset_manager: &mut AssetManager<RuntimeModel>,
) {
    debug!("running render object");
    let model = asset_manager.get(&model.id).unwrap();
    push.set_view_matrix(camera.get_matrix(context.get_screen_size()));
    push.set_model_matrix(transform.build().clone());
    context.draw_mesh(push.to_slice(), &model.mesh);
}
#[system(for_each)]
pub fn render_debug(
    transform: &Transform,
    model: &RuntimeDebugMesh,
    push: &PushBuilder,
    #[resource] settings: &GraphicsSettings,
    #[resource] context: &mut ContextChild,
    #[resource] shader: &ShaderBind,
    #[resource] camera: &Camera,
) {
    push.set_model_matrix(transform.build().clone());
    push.set_view_matrix(camera.get_matrix(context.get_screen_size()));
    context.draw_mesh(push.to_slice(), &model.mesh);
}
#[system(for_each)]
pub fn render_gui(
    transform: &GuiTransform,
    model: &GuiRuntimeModel,
    push: &PushBuilder,
    #[resource] context: &mut ContextChild,
    #[resource] shader: &ShaderBind,
) {
    debug!("running render object");

    push.set_model_matrix(transform.transform.build().clone());
    context.draw_mesh(push.to_slice(), &model.model.mesh);
}
