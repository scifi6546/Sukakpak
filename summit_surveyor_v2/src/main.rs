#![allow(unknown_lints)]
#![allow(clippy::nonstandard_macro_braces)]
mod camera;
mod graph;
mod gui;
mod hud;
mod lift;
mod model;
mod skiier;
mod terrain;
use asset_manager::AssetManager;
use camera::{Camera, Transform};
use gui::EventCollector;
use legion::*;
use model::{Model, ScreenPlane};
use std::{f32, time::Duration};
use sukakpak::{
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3},
    Context, Event, Sukakpak, Texture,
};
use terrain::Terrain;
struct Game {
    world: World,
    resources: Resources,
    game_render_surface: ScreenPlane,
}
pub mod prelude {
    pub use super::camera::{Camera, Transform};
    pub use super::graph::{dijkstra, GraphLayer, GraphNode, GraphType, GraphWeight, Path};
    pub use super::gui::{GuiComponent, GuiItem, TextLabel};
    pub use super::model::Model;
    pub use super::terrain::Terrain;
}

impl sukakpak::Renderable for Game {
    fn init(mut context: Context) -> Self {
        context
            .load_shader("./shaders/test", "world")
            .expect("failed to load");
        context
            .load_shader("./shaders/gui_shader", "gui_shader")
            .expect("failed to load gui shader");
        let mut model_manager: AssetManager<Model> = Default::default();
        let mut texture_manager: AssetManager<Texture> = Default::default();
        let mut resources = Resources::default();
        let mut world = World::default();
        context
            .bind_shader(sukakpak::Bindable::ScreenFramebuffer, "gui_shader")
            .expect("failed to bind");
        Terrain::new_cone(Vector2::new(100, 100), Vector2::new(50.0, 50.0), -1.0, 50.0)
            .insert(&mut world, &mut resources, &mut model_manager, &mut context)
            .expect("failed to build terrain");
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
                    Rgba::from([0, 80, 80, 255]),
                ))
                .expect("failed to build default texture"),
        );

        gui::GuiComponent::insert(
            Box::new(
                gui::VerticalContainer::new(
                    vec![
                        Box::new(
                            gui::GuiSquare::new(
                                Transform::default().set_scale(Vector3::new(0.2, 0.1, 1.0)),
                                default_tex.clone(),
                                hover_tex.clone(),
                                click_tex.clone(),
                                &mut model_manager,
                                &mut texture_manager,
                                &mut context,
                            )
                            .expect("failed to build square"),
                        ),
                        Box::new(
                            gui::GuiSquare::new(
                                Transform::default().set_scale(Vector3::new(0.1, 0.1, 1.0)),
                                default_tex.clone(),
                                hover_tex.clone(),
                                click_tex.clone(),
                                &mut model_manager,
                                &mut texture_manager,
                                &mut context,
                            )
                            .expect("failed to build square"),
                        ),
                        Box::new(
                            gui::VerticalContainer::new(
                                vec![
                                    Box::new(
                                        gui::GuiSquare::new(
                                            Transform::default()
                                                .set_scale(Vector3::new(0.2, 0.1, 1.0)),
                                            default_tex.clone(),
                                            hover_tex.clone(),
                                            click_tex.clone(),
                                            &mut model_manager,
                                            &mut texture_manager,
                                            &mut context,
                                        )
                                        .expect("failed to build square"),
                                    ),
                                    Box::new(
                                        gui::GuiSquare::new(
                                            Transform::default()
                                                .set_scale(Vector3::new(0.1, 0.1, 1.0)),
                                            default_tex,
                                            hover_tex,
                                            click_tex,
                                            &mut model_manager,
                                            &mut texture_manager,
                                            &mut context,
                                        )
                                        .expect("failed to build square"),
                                    ),
                                ],
                                gui::VerticalContainerStyle {
                                    alignment: gui::ContainerAlignment::Center,
                                    padding: 0.01,
                                },
                                Vector3::new(0.0, 0.0, -0.6),
                                &mut context,
                                &mut model_manager,
                                &mut texture_manager,
                            )
                            .expect("failed to create vertical container"),
                        ),
                        Box::new(gui::TextLabel::new(
                "hello world, Here is a loooong paragraph, do you like reading really really really long paragraphs? You know the ones that go on an on forever so long you wonder why the person is still writing. I do so here is one of those loooooong ones."
                    .to_string(),
                            0.003,
                            Transform::default().set_scale(Vector3::new(0.5, 1.0, 1.0)),
                            &mut context,
                            &mut model_manager,
                            &mut texture_manager,
                        ).expect("failed to build text label")),
                    ],
                    gui::VerticalContainerStyle {
                        alignment: gui::ContainerAlignment::Center,
                        padding: 0.01,
                    },
                    Vector3::new(0.0, 0.0, -0.5),
                    &mut context,
                    &mut model_manager,
                    &mut texture_manager,
                )
                .expect("failed to build vertical container"),
            ),
            &mut world,
        )
        .expect("failed to insert?");

        println!("*******************\nBuilding Raw Text\n***************");
        gui::GuiComponent::insert(
            Box::new(gui::TextLabel::new(
                "hello world, Here is a loooong paragraph, do you like reading really really really long paragraphs? You know the ones that go on an on forever so long you wonder why the person is still writing. I do so here is one of those loooooong ones."
                    .to_string(),
                0.006,
                Transform::default()
                    .set_scale(Vector3::new(2.0, 1.0, 1.0))
                    .translate(Vector3::new(0.0, 0.0, 0.0)),
                &mut context,
                &mut model_manager,
                &mut texture_manager
            ).expect("failed to build label")),
            &mut world,
        )
        .expect("failed to insert");

        resources.insert(context);
        resources.insert(model_manager);
        resources.insert(texture_manager);
        Schedule::builder()
            .add_system(lift::insert_lift_system())
            .add_system(hud::build_hud_system())
            .build()
            .execute(&mut world, &mut resources);
        for x in 0..10 {
            for y in 0..1 {
                skiier::Skiier::insert(Vector2::new(x, y), &mut world, &mut resources)
                    .expect("failed to build skiier");
            }
        }
        resources.insert(
            Camera::default()
                .set_translation(Vector3::new(0.0, 2.0, 0.0))
                .set_yaw(f32::consts::PI / 2.0),
        );
        resources.insert(EventCollector::default());
        let game_render_surface = model::build_screen_plane(
            &mut resources.get_mut().unwrap(),
            Vector2::new(1000, 1000),
            0.9,
        )
        .expect("faled to create render surface");
        Self {
            world,
            resources,
            game_render_surface,
        }
    }
    fn render_frame(&mut self, events: &[Event], mut context: Context, delta_time: Duration) {
        self.resources.insert(delta_time);
        self.process_events(delta_time, events);

        context
            .bind_framebuffer(sukakpak::Bindable::UserFramebuffer(
                &self.game_render_surface.framebuffer,
            ))
            .expect("failed to bind");
        let mut game_renderng_schedule = Schedule::builder()
            .add_system(skiier::skiier_system())
            .add_system(hud::update_time_system())
            .add_system(skiier::skiier_path_system())
            .add_system(gui::event::send_events_system())
            .add_system(gui::react_events_system())
            .add_system(model::render_model_system())
            .add_system(terrain_camera_system())
            .build();
        game_renderng_schedule.execute(&mut self.world, &mut self.resources);
        context
            .bind_framebuffer(sukakpak::Bindable::ScreenFramebuffer)
            .expect("failed to bind");
        context
            .draw_mesh(
                Transform::default()
                    .set_translation(Vector3::new(0.0, 0.0, -0.5))
                    .to_bytes(),
                &self.game_render_surface.mesh,
            )
            .expect("failed to draw screen surface");
        let mut gui_rendering_schedule = Schedule::builder()
            //.add_system(gui::render_gui_component_system())
            //.add_system(hud::render_hud_system())
            .build();
        gui_rendering_schedule.execute(&mut self.world, &mut self.resources);
        self.resources.get_mut::<EventCollector>().unwrap().clear();
    }
}
impl Game {
    pub fn process_events(&mut self, delta_time: Duration, events: &[Event]) {
        self.resources
            .get_mut::<EventCollector>()
            .expect("failed to get event collector")
            .process_events(delta_time, events);
    }
}
fn main() {
    Sukakpak::new::<Game>(sukakpak::CreateInfo {
        default_size: Vector2::new(1000, 1000),
        name: "Summit Surveyor".to_string(),
    });
}
#[system]
pub fn terrain_camera(#[resource] events: &mut EventCollector, #[resource] camera: &mut Camera) {
    if events.keycodes_down.contains(&30) {
        *camera = camera.clone().translate(Vector3::new(-0.01, 0.0, 0.0))
    }

    if events.keycodes_down.contains(&32) {
        *camera = camera.clone().translate(Vector3::new(0.01, 0.0, 0.0))
    }

    if events.keycodes_down.contains(&31) {
        *camera = camera.clone().translate(Vector3::new(0.0, 0.0, -0.01))
    }
    if events.keycodes_down.contains(&17) {
        *camera = camera.clone().translate(Vector3::new(0.0, 0.0, 0.01))
    }
    if events.left_mouse_down {
        *camera.yaw() += events.mouse_delta_pos.x * events.delta_time.as_secs_f32() * 1000.0;
        *camera.pitch() += events.mouse_delta_pos.y * events.delta_time.as_secs_f32() * 1000.0;
    }
}
