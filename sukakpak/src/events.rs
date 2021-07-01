use winit::event::WindowEvent as WinitEvent;
pub enum Event {}
pub struct EventCollector {}
impl EventCollector {
    pub fn new() -> Self {
        todo!()
    }
    pub fn push_event(&mut self, event: WinitEvent) {
        todo!()
    }
    pub fn pull_events(&mut self) -> Vec<Event> {
        todo!()
    }
}
