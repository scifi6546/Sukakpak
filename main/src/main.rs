use sukakpak::{nalgebra::Vector2, CreateInfo, Sukakpak};

mod clonecraft;
fn main() {
    Sukakpak::new::<clonecraft::CloneCraft>(CreateInfo {
        default_size: Vector2::new(800, 800),
        name: "clonecraft".to_string(),
    });
}
