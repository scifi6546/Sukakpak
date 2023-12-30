use std::path::PathBuf;
use sukakpak::{nalgebra::Vector2, CreateInfo, Sukakpak};

mod clonecraft;
fn main() {
    sukakpak::run::<clonecraft::CloneCraft>(CreateInfo {
        window_id: "canvas".to_string(),
        default_size: Vector2::new(800, 800),
        name: "clonecraft".to_string(),
        vulkan_sdk_path: None,
        //vulkan_sdk_path: Some(PathBuf::from("C:/VulkanSDK/1.3.268.0/Lib")),
    });
}
