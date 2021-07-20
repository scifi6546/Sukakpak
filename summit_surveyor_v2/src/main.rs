mod camera;
mod model;
use camera::Camera;
use legion::*;
use model::{RenderingCtx, Terrain};
use std::{cell::RefCell, rc::Rc};
use sukakpak::{nalgebra::Vector2, Context, Event, Sukakpak};
struct Game {
    world: World,
    resources: Resources,
}
pub mod prelude {
    pub use super::camera::{Camera, Transform};
    pub use super::model::Terrain;
}

impl sukakpak::Renderable for Game {
    fn init(context: Rc<RefCell<Context>>) -> Self {
        context
            .borrow_mut()
            .load_shader("./shaders/shaders/test", "world")
            .expect("failed to load");
        let mut resources = Resources::default();
        let mut world = World::default();
        Terrain::new_flat(Vector2::new(100, 100))
            .insert(&mut world, &context)
            .expect("failed to build terrain");

        resources.insert(RenderingCtx::new(&context));
        resources.insert(Camera::default());
        Self { world, resources }
    }
    fn render_frame(
        &mut self,
        events: &[Event],
        _context: Rc<RefCell<Context>>,
        delta_time_ms: f32,
    ) {
        let mut schedule = Schedule::builder()
            .add_system(model::render_model_system())
            .build();
        schedule.execute(&mut self.world, &mut self.resources);
    }
}
fn main() {
    Sukakpak::new::<Game>(sukakpak::CreateInfo {
        default_size: Vector2::new(1000, 1000),
        name: "Summit Surveyor".to_string(),
    });
}
