mod asset_manager;
mod bindable;
mod camera;
mod graph;
mod graphics_engine;
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
use log::{debug, info};
use sukakpak::{
    nalgebra::{Matrix4, Vector2, Vector3, Vector4},
    BoundFramebuffer, ContextChild, Event, Framebuffer,
};
use texture::RGBATexture;
use transform::Transform;
mod events;
use bindable::Bindable;
use camera::Camera;

use asset_manager::AssetManager;
use graph::graph_debug;
//
use graphics_system::{insert_terrain, GraphicsSettings, RuntimeModel};
use gui::GuiModel;
use legion::*;
use lift::insert_lift;
use terrain::Terrain;
pub mod prelude {
    pub use super::asset_manager::AssetManager;
    pub use super::camera::Camera;
    pub use super::graph::{
        dijkstra, FollowPath, GraphLayer, GraphLayerList, GraphWeight, GridNode, LiftLayer, Node,
        NodeFloat, Path,
    };
    pub use super::graphics_engine::{ItemDesc, Mesh, Vertex};
    pub use super::transform::Transform;
    pub use sukakpak::nalgebra as na;
    pub type ShaderBind = super::Bindable<String>;
    pub use super::events::{Event, MouseButton};
    pub use super::graphics_system::{RuntimeDebugMesh, RuntimeModel, RuntimeModelId};
    pub use super::grid::Grid;
    pub use super::gui::{GuiModel, GuiRuntimeModel, GuiTransform};
    pub use super::model::Model;
    pub use super::terrain::Terrain;
    pub use super::texture::RGBATexture as Texture;
}
use prelude::ShaderBind;
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct PushBuilder {
    sun_direction: Vector3<f32>,
    sun_color: Vector4<f32>,
    view_matrix: Matrix4<f32>,
    is_built: bool,
}
impl PushBuilder {
    pub fn new(sun_direction: Vector3<f32>, sun_color: Vector4<f32>) -> Self {
        Self {
            sun_direction,
            sun_color,
            view_matrix: Matrix4::zeros(),
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
    pub fn build(&mut self, view_matrix: Matrix4<f32>) {
        self.view_matrix = view_matrix;
        self.is_built = true;
    }
}
pub struct Game {
    world: World,
    resources: Resources,
    world_framebuffer: Framebuffer,
    world_render_surface: RuntimeModel,
    push_builder: PushBuilder,
}
impl sukakpak::Renderable for Game {
    fn init(context: &mut ContextChild) -> Self {
        utils::set_panic_hook();
        let mut resources = Resources::default();
        let mut world = World::default();
        let mut shader_bind = Bindable::default();
        let mut model_manager = AssetManager::default();
        shader_bind.insert("world", "world".to_string());
        shader_bind.bind("world");
        context
            .bind_shader(&BoundFramebuffer::ScreenFramebuffer, shader_bind.get_bind())
            .ok()
            .unwrap();
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

        GuiModel::simple_box(box_transform).insert(&mut world, context, &shader_bind.get_bind());
        insert_terrain(
            Terrain::new_cone(Vector2::new(20, 20), Vector2::new(10.0, 10.0), 5.0, -1.0),
            &mut world,
            context,
            &mut model_manager,
            &shader_bind.get_bind(),
        );
        insert_lift(
            &mut world,
            context,
            &mut model_manager,
            &shader_bind,
            Vector2::new(0, 0),
            Vector2::new(10, 10),
        );
        println!("inserted lift");
        let mut world_framebuffer_texture = context.build_texture(&RGBATexture::constant_color(
            Vector4::new(0, 0, 0, 0),
            screen_size,
        ));
        println!("built world frame_buffer_texture");

        let fb_mesh = context.build_mesh(Mesh::plane(), &shader_bind.get_bind())?;
        let world_framebuffer =
            webgl.build_framebuffer(&mut world_framebuffer_texture, &mut world_depth_texture)?;
        println!("built world framebuffer");
        let world_render_surface = RuntimeModel {
            mesh: fb_mesh,
            texture: world_framebuffer_texture,
        };

        info!("building skiiers");
        println!("building skiiers");
        for i in 0..10 {
            skiier::build_skiier(&mut world, &mut context, &shader_bind, Vector2::new(i, 0))?;
        }
        info!("done building skiiers");
        resources.insert(context);
        resources.insert(shader_bind);
        resources.insert(GraphicsSettings {
            screen_size: screen_size.clone(),
        });
        resources.insert(Camera::new(Vector3::new(0.0, 0.0, 0.0), 20.0, 1.0, 1.0));
        let (egui_context, egui_adaptor) = gui::init_gui(screen_size);
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
        };
        info!("built game successfully");
        g
    }
    fn render_frame(&mut self, events: &[Event], context: &mut ContextChild) {
        {
            let context = &mut self.resources.get_mut().unwrap();
            gui::insert_ui(context);
        }
        {
            let camera: &mut Camera = &mut self.resources.get_mut().unwrap();
            for e in events.iter() {
                match e {
                    Event::MouseMoved { position } => {
                        if buttons_pressed.contains(&MouseButton::RightClick) {
                            camera.rotate_phi(delta_x * 0.001 * delta_time_ms);
                            camera.rotate_theta(delta_y * 0.001 * delta_time_ms);
                        }
                    }
                    Event::WindowResized { new_size } => {
                        let shader: &mut ShaderBind = &mut self.resources.get_mut().unwrap();
                        shader.bind("screen");

                        let settings: &mut GraphicsSettings =
                            &mut self.resources.get_mut().unwrap();
                        settings.screen_size = new_size.clone();
                        context.delete_texture(&mut self.world_render_surface.texture);
                        context
                            .delete_mesh(&mut self.world_render_surface.mesh)
                            .expect("failed to delete framebuffer mesh");
                        context
                            .delete_framebuffer(&mut self.world_framebuffer)
                            .expect("deleted old framebuffer");
                        let mut world_framebuffer_texture = context
                            .build_texture(
                                RGBATexture::constant_color(
                                    Vector4::new(0, 0, 0, 0),
                                    new_size.clone(),
                                ),
                                &shader.get_bind(),
                            )
                            .expect("failed to build new texture");

                        let fb_mesh = context
                            .build_mesh(Mesh::plane(), &shader.get_bind())
                            .expect("failed to build mesh");
                        self.world_framebuffer = gl
                            .build_framebuffer(
                                &mut world_framebuffer_texture,
                                &mut world_depth_texture,
                            )
                            .expect("failed to build framebuffer");
                        self.world_render_surface = RuntimeModel {
                            mesh: fb_mesh,
                            texture: world_framebuffer_texture,
                        };
                    }
                    Event::CameraMove { direction } => camera.translate(&(0.1 * direction)),
                    Event::Scroll {
                        delta_y,
                        delta_time_ms,
                    } => {
                        camera.update_radius(0.0000001 * delta_y * delta_time_ms);
                        debug!("zoomed");
                    }
                    _ => (),
                }
            }

            //binding to world framebuffer and rendering to it

            context.bind_framebuffer(&BoundFramebuffer::UserFramebuffer(self.world_framebuffer));

            let shader: &mut ShaderBind = &mut self.resources.get_mut().unwrap();
            shader.bind("world");
            context.bind_shader(shader.get_bind()).ok().unwrap();
        }
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
        {
            let gl: &mut RenderingContext = &mut self.resources.get_mut().unwrap();
            gl.clear_depth();
        }
        let mut schedule = Schedule::builder()
            .add_system(graphics_system::render_debug_system())
            .build();
        schedule.execute(&mut self.world, &mut self.resources);
        {
            //binding to world framebuffer and rendering to it

            let gl: &mut RenderingContext = &mut self.resources.get_mut().unwrap();
            let shader: &mut ShaderBind = &mut self.resources.get_mut().unwrap();
            shader.bind("screen");
            gl.bind_default_framebuffer();
            //getting screen shader
            gl.bind_shader(shader.get_bind()).ok().unwrap();
            gl.send_view_matrix(Matrix4::identity(), shader.get_bind());
            gl.send_model_matrix(Matrix4::identity(), shader.get_bind());
            gl.clear_screen(Vector4::new(0.2, 0.2, 0.2, 1.0));
            gl.bind_texture(&self.world_render_surface.texture, shader.get_bind());
            gl.draw_mesh(&self.world_render_surface.mesh);

            //binding and drawing gui shader
            shader.bind("gui");
            gl.bind_shader(shader.get_bind()).ok().unwrap();
            {
                let egui_context = &mut self.resources.get_mut().unwrap();
                let egui_adaptor = &mut self.resources.get_mut().unwrap();
                let settings: &GraphicsSettings = &self.resources.get().unwrap();
                gui::draw_gui(
                    egui_context,
                    &events,
                    gl,
                    shader,
                    egui_adaptor,
                    settings.screen_size,
                )
                .expect("successfully drew");
            }
            shader.bind("screen");
            //getting screen shader
            gl.bind_shader(shader.get_bind()).ok().unwrap();
        }
        let mut gui_schedule = Schedule::builder()
            .add_system(graphics_system::render_gui_system())
            .build();
        gui_schedule.execute(&mut self.world, &mut self.resources);
    }
}
