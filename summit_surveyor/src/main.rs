mod asset_manager;
mod bindable;
mod camera;
mod graph;
mod graphics_system;
mod grid;
mod gui;
mod lift;
mod model;
mod skiier;
mod terrain;
mod texture;
mod transform;
mod utils;
use bindable::Bindable;
use camera::Camera;
use log::info;
use sukakpak::{
    nalgebra::{Matrix4, Vector2, Vector3, Vector4},
    BoundFramebuffer, Context, Event, Framebuffer, MeshAsset, MeshTexture, MouseButton,
};
use transform::Transform;

use asset_manager::AssetManager;
use graph::graph_debug;
//
use graphics_system::{insert_terrain, GraphicsSettings, RuntimeModel};
use gui::GuiModel;
use legion::*;
use lift::insert_lift;
use std::{cell::RefCell, rc::Rc};
use terrain::Terrain;
mod prelude {
    use std::{cell::RefCell, rc::Rc};
    pub struct RenderingCtx(pub Rc<RefCell<sukakpak::Context>>);
    unsafe impl Send for RenderingCtx {}
    unsafe impl Sync for RenderingCtx {}
    pub use super::asset_manager::AssetManager;
    pub use super::camera::Camera;
    pub use super::graph::{
        dijkstra, FollowPath, GraphLayer, GraphLayerList, GraphWeight, GridNode, LiftLayer, Node,
        NodeFloat, Path,
    };
    pub use super::transform::Transform;
    pub use super::PushBuilder;
    pub use sukakpak::{
        anyhow::Result, image, nalgebra as na, Context, EasyMesh, EasyMeshVertex as Vertex,
        MeshAsset, VertexComponent, VertexLayout,
    };

    pub type ShaderBind = super::Bindable<String>;
    pub use super::graphics_system::{RuntimeDebugMesh, RuntimeModel, RuntimeModelId};
    pub use super::grid::Grid;
    pub use super::gui::{GuiModel, GuiRuntimeModel, GuiTransform};
    pub use super::model::Model;
    pub use super::terrain::Terrain;
    pub use super::texture::RGBATexture as Texture;
    pub use sukakpak::Event;
}
use prelude::ShaderBind;
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct PushBuilder {
    sun_direction: Vector3<f32>,
    sun_color: Vector4<f32>,
    out_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    model_matrix: Matrix4<f32>,

    is_built: bool,
}
impl PushBuilder {
    pub fn new(sun_direction: Vector3<f32>, sun_color: Vector4<f32>) -> Self {
        Self {
            sun_direction,
            sun_color,
            out_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            model_matrix: Matrix4::identity(),
            is_built: false,
        }
    }
    pub fn to_slice(&mut self) -> &[u8] {
        assert!(self.is_built);
        unsafe {
            std::slice::from_raw_parts(
                self.sun_direction.as_ptr() as *const u8,
                std::mem::size_of::<Vector3<f32>>()
                    + std::mem::size_of::<Vector4<f32>>()
                    + std::mem::size_of::<Matrix4<f32>>(),
            )
        }
    }
    pub fn set_view_matrix(&mut self, view_matrix: Matrix4<f32>) {
        self.view_matrix = view_matrix;
        self.out_matrix = self.view_matrix * self.model_matrix;
    }
    pub fn set_model_matrix(&mut self, model_matrix: Matrix4<f32>) {
        self.model_matrix = model_matrix;
        self.out_matrix = self.view_matrix * self.model_matrix;
    }
    pub fn make_identity(&mut self) {
        self.view_matrix = Matrix4::identity();
        self.model_matrix = Matrix4::identity();
    }
}
use prelude::RenderingCtx;
pub struct Game {
    world: World,
    resources: Resources,
    world_framebuffer: Framebuffer,
    world_render_surface: RuntimeModel,
    push_builder: PushBuilder,
    last_mouse_pos: Vector2<f32>,
}
impl sukakpak::Renderable for Game {
    fn init(context: Rc<RefCell<Context>>) -> Self {
        utils::set_panic_hook();
        let mut rendering_ctx = RenderingCtx(Rc::clone(&context));
        let mut resources = Resources::default();
        let mut world = World::default();
        let mut shader_bind = Bindable::default();
        let mut model_manager = AssetManager::default();
        let screen_size = context.borrow_mut().get_screen_size();
        context
            .borrow_mut()
            .load_shader("shaders/world", "world")
            .expect("failed to load");
        shader_bind.insert("world", "world".to_string());
        shader_bind.bind("world");
        {
            let mut ctx_ref = context.borrow_mut();
            ctx_ref
                .bind_shader(&BoundFramebuffer::ScreenFramebuffer, shader_bind.get_bind())
                .ok()
                .unwrap();
        }
        let push_builder = PushBuilder::new(
            Vector3::new(1.0, -1.0, 0.0).normalize(),
            Vector4::new(1.0, 1.0, 1.0, 1.0),
        );
        shader_bind.insert("screen", "screen".to_string());
        shader_bind.insert("gui", "gui".to_string());
        shader_bind.bind("world");
        let mut box_transform = Transform::default();
        box_transform.set_scale(Vector3::new(0.1, 0.1, 0.1));
        box_transform.translate(Vector3::new(-0.5, -0.5, 0.0));

        GuiModel::simple_box(box_transform)
            .insert(&mut world, &mut rendering_ctx)
            .expect("failed to build gui model");
        insert_terrain(
            Terrain::new_cone(Vector2::new(20, 20), Vector2::new(10.0, 10.0), 5.0, -1.0),
            &mut world,
            &mut rendering_ctx,
            &mut model_manager,
        )
        .expect("failed to build terrain model");
        insert_lift(
            &mut world,
            &mut rendering_ctx,
            &mut model_manager,
            Vector2::new(0, 0),
            Vector2::new(10, 10),
        )
        .expect("failed to build lift model");
        println!("inserted lift");
        let world_framebuffer = {
            let mut ctx_ref = context.borrow_mut();
            let screen_size = ctx_ref.get_screen_size();
            ctx_ref
                .build_framebuffer(screen_size)
                .expect("failed to build framebuffer")
        };

        let fb_mesh = context.borrow_mut().build_mesh(
            MeshAsset::new_plane(),
            MeshTexture::Framebuffer(world_framebuffer),
        );

        println!("built world framebuffer");
        let world_render_surface = RuntimeModel { mesh: fb_mesh };

        info!("building skiiers");
        println!("building skiiers");
        for i in 0..10 {
            skiier::build_skiier(&mut world, &mut rendering_ctx, Vector2::new(i, 0))
                .expect("failed to build skiiers");
        }
        info!("done building skiiers");
        resources.insert(shader_bind);
        resources.insert(GraphicsSettings {});
        resources.insert(Camera::new(Vector3::new(0.0, 0.0, 0.0), 20.0, 1.0, 1.0));
        let (egui_context, egui_adaptor) = gui::init_gui(screen_size);
        resources.insert(RenderingCtx(context.clone()));
        resources.insert(egui_context);
        resources.insert(egui_adaptor);
        resources.insert(model_manager);
        // gui::insert_ui(&mut egui_context);
        info!("context created");
        info!("inserted ui");
        let g = Game {
            world,
            resources,
            world_framebuffer,
            world_render_surface,
            push_builder,
            last_mouse_pos: Vector2::new(0.0, 0.0),
        };
        info!("built game successfully");
        g
    }
    fn render_frame(
        &mut self,
        events: &[Event],
        context: Rc<RefCell<Context>>,
        delta_time_ms: f32,
    ) {
        {
            let context = &mut self.resources.get_mut().unwrap();
            gui::insert_ui(context);
        }
        {
            let buttons_pressed = vec![];
            let camera: &mut Camera = &mut self.resources.get_mut().unwrap();
            for e in events.iter() {
                match e {
                    Event::MouseMoved { position, .. } => {
                        if buttons_pressed.contains(&MouseButton::Right) {
                            let delta = position - self.last_mouse_pos;
                            camera.rotate_phi(delta.x * 0.001 * delta_time_ms);
                            camera.rotate_theta(delta.y * 0.001 * delta_time_ms);
                        }
                        self.last_mouse_pos = *position;
                    }
                    Event::ScrollContinue { delta } => {
                        camera.update_radius(0.001 * delta.y() * delta_time_ms);
                    }
                    _ => (),
                }
            }

            //WORKS HERE
            //
            //
            //binding to world framebuffer and rendering to it
            {
                let mut ctx_ref = context.borrow_mut();
                ctx_ref
                    .bind_framebuffer(&BoundFramebuffer::UserFramebuffer(self.world_framebuffer))
                    .expect("failed to bind");

                let shader: &mut ShaderBind = &mut self.resources.get_mut().unwrap();
                shader.bind("world");
                ctx_ref
                    .bind_shader(
                        &BoundFramebuffer::UserFramebuffer(self.world_framebuffer),
                        shader.get_bind(),
                    )
                    .expect("failed to bind");
            }
        }
        //errors out before here
        return;
        //game logic
        let mut schedule = Schedule::builder()
            .add_system(skiier::follow_path_system())
            .build();
        schedule.execute(&mut self.world, &mut self.resources);
        let mut schedule = Schedule::builder()
            .add_system(graphics_system::render_object_system())
            .build();
        {
            let ctx: &mut egui::CtxRef = &mut self.resources.get_mut().unwrap();
            graph_debug::terrain_debug_window(&self.world, ctx);
            skiier::draw_skiiers(&self.world, ctx);
        }
        schedule.execute(&mut self.world, &mut self.resources);
        let mut schedule = Schedule::builder()
            .add_system(graphics_system::render_debug_system())
            .build();
        schedule.execute(&mut self.world, &mut self.resources);
        {
            //binding to world framebuffer and rendering to it

            let shader: &mut ShaderBind = &mut self.resources.get_mut().unwrap();
            shader.bind("screen");
            //failed to draw here
            context
                .borrow_mut()
                .bind_framebuffer(&BoundFramebuffer::ScreenFramebuffer)
                .expect("failed to bind");
            //getting screen shader
            context
                .borrow_mut()
                .bind_shader(&BoundFramebuffer::ScreenFramebuffer, shader.get_bind())
                .ok()
                .unwrap();
            self.push_builder.make_identity();

            context
                .borrow_mut()
                .draw_mesh(
                    self.push_builder.to_slice(),
                    &self.world_render_surface.mesh,
                )
                .expect("failed to draw");

            //binding and drawing gui shader
            shader.bind("gui");
            context
                .borrow_mut()
                .bind_shader(&BoundFramebuffer::ScreenFramebuffer, shader.get_bind())
                .expect("failed to bind gui shader");
            {
                let screen_size = context.borrow_mut().get_screen_size();
                let egui_context = &mut self.resources.get_mut().unwrap();
                let egui_adaptor = &mut self.resources.get_mut().unwrap();
                let mut rendering_ctx = RenderingCtx(context.clone());
                gui::draw_gui(
                    egui_context,
                    events,
                    &mut rendering_ctx,
                    egui_adaptor,
                    screen_size,
                )
                .expect("successfully drew");
            }
            shader.bind("screen");
            //getting screen shader
            context
                .borrow_mut()
                .bind_shader(&BoundFramebuffer::ScreenFramebuffer, shader.get_bind())
                .expect("failed to bind screen");
        }
        let mut gui_schedule = Schedule::builder()
            .add_system(graphics_system::render_gui_system())
            .build();
        gui_schedule.execute(&mut self.world, &mut self.resources);
    }
}
fn main() {
    sukakpak::Sukakpak::new::<Game>(sukakpak::CreateInfo {
        name: "Summit Surveyor".to_string(),
        default_size: Vector2::new(800, 800),
    });
}
