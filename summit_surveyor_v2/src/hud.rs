use super::prelude::{GuiItem, TextLabel, Transform};
use legion::systems::CommandBuffer;
use legion::*;
use std::time::Duration;
use sukakpak::{nalgebra::Vector3, Context};

pub struct Hud {
    time: TextLabel,
}
impl Hud {
    const TEXT_SIZE: f32 = 0.006;
    fn get_transform() -> Transform {
        Transform::default()
            .set_scale(Vector3::new(0.1, 1.0, 1.0))
            .translate(Vector3::new(-0.8, -0.8, 0.0))
    }
}
#[system]
pub fn build_hud(command_buffer: &mut CommandBuffer, #[resource] graphics: &mut Context) {
    command_buffer.push((
        Hud {
            time: TextLabel::new(
                "f".to_string(),
                Hud::TEXT_SIZE,
                Hud::get_transform(),
                graphics,
            )
            .expect("failed to build text label"),
        },
        (),
    ));
}
#[system(for_each)]
pub fn update_time(
    hud: &mut Hud,
    #[resource] graphics: &mut Context,
    #[resource] duration: &Duration,
) {
    *hud = Hud {
        time: TextLabel::new(
            format!("{} fps", 1.0 / duration.as_secs_f32()),
            Hud::TEXT_SIZE,
            Hud::get_transform(),
            graphics,
        )
        .expect("failed to build time label"),
    };
}

#[system(for_each)]
pub fn render_hud(hud: &mut Hud, #[resource] graphics: &mut Context) {
    hud.time.render(Transform::default(), graphics);
}
