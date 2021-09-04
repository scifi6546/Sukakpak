use std::{cell::RefCell, rc::Rc, time::Duration};
use sukakpak::{Context, Event};
pub struct CloneCraft {}
impl sukakpak::Renderable for CloneCraft {
    fn init(_context: Rc<RefCell<Context>>) -> Self {
        Self {}
    }
    fn render_frame(
        &mut self,
        events: &[Event],
        _context: Rc<RefCell<Context>>,
        _delta_time: Duration,
    ) {
        println!("ran frame");
        for e in events.iter() {}
    }
}
