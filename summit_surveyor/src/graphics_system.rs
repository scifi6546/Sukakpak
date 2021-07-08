use super::prelude::{
    AssetManager, Camera, ContextChild, GuiRuntimeModel, GuiTransform, Model, ShaderBind, Terrain,Shader
    Transform,
};
use legion::*;
use log::debug;
use std::cell::RefCell;
use sukakpak::nalgebra::Vector2;
use sukakpak::MeshTexture;
pub struct RuntimeModel {
    pub mesh: RuntimeMesh,
    pub texture: MeshTexture,
}
/// Used for printing debug info
pub struct RuntimeDebugMesh {
    mesh: RuntimeMesh,
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
    pub fn new(
        model: &Model,
        context: &mut ContextChild,
        bound_shader: &Shader,
    ) -> Result<Self, ErrorType> {
        let texture = context.build_texture(&model.texture.into())?;
        let mesh = context.build_mesh(model.mesh.clone(), texture);
        Ok(Self { mesh, texture })
    }
}
impl RuntimeDebugMesh {
    pub fn new(
        model: &Model,
        context: &mut ContextChild,
        bound_shader: &Shader,
    ) -> Result<Self, ErrorType> {
        let texture = context.build_texture(&model.texture.into())?;
        let mesh = context.build_mesh(model.mesh, texture);
        Ok(Self { mesh })
    }
}
pub fn insert_terrain(
    terrain: Terrain,
    world: &mut World,
    graphics: &mut RenderingContext,
    asset_manager: &mut AssetManager<RuntimeModel>,
    bound_shader: &Shader,
) -> Result<(), ErrorType> {
    let model = terrain.model();
    let transform = model.transform.clone();
    asset_manager.get_or_create(
        "game_terrain",
        RuntimeModel::new(&model, graphics, bound_shader).expect("created model"),
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
    #[resource] settings: &GraphicsSettings,
    #[resource] webgl: &mut RenderingContext,
    #[resource] shader: &ShaderBind,
    #[resource] camera: &Camera,
    #[resource] asset_manager: &mut AssetManager<RuntimeModel>,
) {
    debug!("running render object");
    let model = asset_manager.get(&model.id).unwrap();
    webgl.bind_texture(&model.texture, shader.get_bind());
    webgl.send_view_matrix(camera.get_matrix(settings.screen_size), shader.get_bind());
    webgl.send_model_matrix(transform.build().clone(), shader.get_bind());
    webgl.draw_mesh(&model.mesh);
}
#[system(for_each)]
pub fn render_debug(
    transform: &Transform,
    model: &RuntimeDebugMesh,
    #[resource] settings: &GraphicsSettings,
    #[resource] webgl: &mut RenderingContext,
    #[resource] shader: &ShaderBind,
    #[resource] camera: &Camera,
) {
    webgl.send_model_matrix(transform.build().clone(), shader.get_bind());
    webgl.send_view_matrix(camera.get_matrix(settings.screen_size), shader.get_bind());
    webgl.draw_lines(&model.mesh);
}
#[system(for_each)]
pub fn render_gui(
    transform: &GuiTransform,
    model: &GuiRuntimeModel,
    #[resource] webgl: &mut RenderingContext,
    #[resource] shader: &ShaderBind,
) {
    debug!("running render object");
    webgl.bind_texture(&model.model.texture, shader.get_bind());
    webgl.send_model_matrix(transform.transform.build().clone(), shader.get_bind());
    webgl.draw_mesh(&model.model.mesh);
}
