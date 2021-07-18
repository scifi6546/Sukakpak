use super::prelude;
use super::prelude::{
    na::{Vector2, Vector4},
    Event, MeshAsset, Model, RenderingCtx, Result, RuntimeModel, Texture, Transform,
};
use legion::*;
mod egui_integration;
use egui::CtxRef;
use egui_integration::draw_egui;
pub use egui_integration::EguiRawInputAdaptor;
pub struct GuiRuntimeModel {
    pub model: RuntimeModel,
}
pub struct GuiTransform {
    pub transform: Transform,
}
#[derive(Clone)]
pub struct GuiModel {
    model: Model,
}
impl GuiModel {
    pub fn simple_box(transform: Transform) -> Self {
        Self {
            model: Model {
                mesh: MeshAsset::new_plane(),
                texture: Texture::constant_color(
                    Vector4::new(255 / 10, 255 / 2, 255 / 2, 255),
                    Vector2::new(100, 100),
                ),
                transform,
            },
        }
    }
    pub fn insert(&self, world: &mut World, ctx: &mut RenderingCtx) -> Result<Entity> {
        let transform = GuiTransform {
            transform: self.model.transform.clone(),
        };
        let model = RuntimeModel::new(&self.model, ctx);
        Ok(world.push((transform, GuiRuntimeModel { model })))
    }
}
trait App: std::fmt::Debug {}
#[derive(Debug, Clone)]
struct Test<T: Clone> {
    t: T,
}
#[allow(unused_must_use)]
pub fn init_gui(screen_size: Vector2<u32>) -> (CtxRef, EguiRawInputAdaptor) {
    let mut ctx = CtxRef::default();
    let mut adaptor = EguiRawInputAdaptor::default();
    ctx.begin_frame(adaptor.process_events(&vec![], screen_size));
    //not painting because it is just in the init phase
    ctx.end_frame();
    (ctx, adaptor)
}
pub fn draw_gui(
    context: &mut CtxRef,
    input: &[Event],
    ctx: &mut RenderingCtx,
    adaptor: &mut EguiRawInputAdaptor,
    screen_size: Vector2<u32>,
) -> Result<()> {
    context.begin_frame(adaptor.process_events(input, screen_size));
    let (_, commands) = context.end_frame();
    let paint_jobs = context.tessellate(commands);
    draw_egui(&paint_jobs, &context.texture(), ctx, &screen_size)?;
    Ok(())
}
pub fn insert_ui(context: &mut CtxRef) {
    egui::Window::new("dfsadfas").show(context, |ui| {
        ui.label("Can I Read?");
    });
}
