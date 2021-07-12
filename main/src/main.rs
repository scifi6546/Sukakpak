pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

use sukakpak::{nalgebra::Vector2, CreateInfo, Sukakpak};

mod clonecraft;
fn main() {
    Sukakpak::new::<clonecraft::CloneCraft>(CreateInfo {
        default_size: Vector2::new(800, 800),
        name: "clonecraft".to_string(),
    });
}
