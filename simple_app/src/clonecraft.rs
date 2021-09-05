use std::time::Duration;
use sukakpak::{Context, Event};
pub struct CloneCraft {}
impl sukakpak::Renderable for CloneCraft {
    fn init(_context: Context) -> Self {
        Self {}
    }
    fn render_frame(&mut self, _events: &[Event], _context: Context, _delta_time: Duration) {}
}
