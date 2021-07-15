use sukakpak::nalgebra::Vector2;
use summit_surveyor::Game;
fn main() {
    sukakpak::Sukakpak::new::<Game>(sukakpak::CreateInfo {
        default_size: Vector2::new(800, 800),
        name: "game".to_string(),
    });
}
