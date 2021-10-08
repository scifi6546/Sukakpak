use sukakpak::{nalgebra::Vector2, run, CreateInfo, Sukakpak};

mod clonecraft;
fn main() {
    run::<clonecraft::CloneCraft>(CreateInfo {
        default_size: Vector2::new(800, 800),
        name: "clonecraft".to_string(),
        window_id: "canvas".to_string(),
    });
}
