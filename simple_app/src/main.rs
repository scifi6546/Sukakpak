use sukakpak::{nalgebra::Vector2, CreateInfo, Sukakpak};

mod clonecraft;
fn main() {
    sukakpak::run::<clonecraft::CloneCraft>(CreateInfo {
        window_id: "canvas".to_string(),
        default_size: Vector2::new(800, 800),
        name: "clonecraft".to_string(),
    });
}
