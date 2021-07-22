mod camera;
mod gui;
mod model;
use camera::{Camera, Transform};
use gui::EventCollector;
use legion::*;
use model::{RenderingCtx, ScreenPlane, Terrain};
use std::{cell::RefCell, rc::Rc};
use sukakpak::{
    nalgebra::{Vector2, Vector3},
    Context, Event, Sukakpak,
};
struct Game {
    world: World,
    resources: Resources,
    game_render_surface: ScreenPlane,
}
pub mod prelude {
    pub use super::camera::{Camera, Transform};
    pub use super::model::{RenderingCtx, Terrain};
}

impl sukakpak::Renderable for Game {
    fn init(context: Rc<RefCell<Context>>) -> Self {
        context
            .borrow_mut()
            .load_shader("./shaders/test", "world")
            .expect("failed to load");
        context
            .borrow_mut()
            .load_shader("./shaders/gui_shader", "gui_shader")
            .expect("failed to load gui shader");
        let mut resources = Resources::default();
        let mut world = World::default();
        context
            .borrow_mut()
            .bind_shader(&sukakpak::BoundFramebuffer::ScreenFramebuffer, "gui_shader")
            .expect("failed to bind");
        Terrain::new_flat(Vector2::new(100, 100))
            .insert(&mut world, &context)
            .expect("failed to build terrain");
        model::insert_cube(
            Transform::default()
                .set_scale(Vector3::new(0.2, 0.2, 0.2))
                .translate(Vector3::new(0.0, 0.0, 1.0)),
            &mut world,
            context.clone(),
        )
        .expect("failed to insert");
        gui::GuiSquare::insert(
            Transform::default()
                .set_scale(Vector3::new(0.2, 1.0, 1.0))
                .set_translation(Vector3::new(0.2, 0.0, -0.2)),
            &mut world,
            context.clone(),
        )
        .expect("failed to build gui square");

        resources.insert(RenderingCtx::new(&context));
        resources.insert(Camera::default());
        resources.insert(EventCollector::default());
        let game_render_surface =
            model::build_screen_plane(context.clone(), Vector2::new(1000, 1000), 0.0)
                .expect("faled to create render surface");
        Self {
            world,
            resources,
            game_render_surface,
        }
    }
    fn render_frame(
        &mut self,
        events: &[Event],
        context: Rc<RefCell<Context>>,
        _delta_time_ms: f32,
    ) {
        self.process_events(events);

        context
            .borrow_mut()
            .bind_framebuffer(&sukakpak::BoundFramebuffer::UserFramebuffer(
                self.game_render_surface.framebuffer,
            ))
            .expect("failed to bind");
        let mut game_renderng_schedule = Schedule::builder()
            .add_system(gui::send_events_system())
            .add_system(gui::react_events_system())
            .add_system(model::render_model_system())
            .build();
        game_renderng_schedule.execute(&mut self.world, &mut self.resources);
        context
            .borrow_mut()
            .bind_framebuffer(&sukakpak::BoundFramebuffer::ScreenFramebuffer)
            .expect("failed to bind");
        context
            .borrow_mut()
            .draw_mesh(
                Transform::default()
                    .set_translation(Vector3::new(0.0, 0.0, -0.5))
                    .to_bytes(),
                &self.game_render_surface.mesh,
            )
            .expect("failed to draw screen surface");
        let mut gui_rendering_schedule = Schedule::builder()
            .add_system(gui::render_gui_system())
            .build();
        gui_rendering_schedule.execute(&mut self.world, &mut self.resources);
    }
}
impl Game {
    pub fn process_events(&mut self, events: &[Event]) {
        self.resources
            .get_mut::<EventCollector>()
            .expect("failed to get event collector")
            .process_events(events);
    }
}
fn main() {
    Sukakpak::new::<Game>(sukakpak::CreateInfo {
        default_size: Vector2::new(1000, 1000),
        name: "Summit Surveyor".to_string(),
    });
}
