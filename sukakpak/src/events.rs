use nalgebra::Vector2;
use winit::event::WindowEvent as WinitEvent;
#[derive(Clone, Debug)]
pub enum Event {
    ProgramTermination,
    Resized { new_size: Vector2<u32> },
}
pub struct EventCollector {
    events: Vec<Event>,
    quit: bool,
}
impl EventCollector {
    pub fn new() -> Self {
        Self {
            events: vec![],
            quit: false,
        }
    }
    pub fn push_event(&mut self, event: WinitEvent) {
        match event {
            WinitEvent::Resized(size) => self.events.push(Event::Resized {
                new_size: Vector2::new(size.width, size.height),
            }),
            WinitEvent::Moved(_) => todo!("Handle moved"),
            WinitEvent::CloseRequested => {
                self.quit = true;
            }
            WinitEvent::Destroyed => {
                self.quit = true;
            }
            WinitEvent::DroppedFile(_) => todo!("dropped file"),
            WinitEvent::HoveredFile(_) => todo!("hover file"),
            WinitEvent::HoveredFileCancelled => todo!("hovered file canceled"),
            WinitEvent::ReceivedCharacter(_) => todo!("received character"),
            WinitEvent::Focused(_) => todo!("focused"),
            WinitEvent::KeyboardInput { .. } => todo!("keyboard input"),
            WinitEvent::ModifiersChanged(_) => todo!("Modifiers Changed"),
            WinitEvent::CursorMoved { .. } => todo!("cursor moved"),
            WinitEvent::CursorEntered { .. } => todo!("cursor entered"),
            WinitEvent::CursorLeft { .. } => todo!("cursor left"),
            WinitEvent::MouseWheel { .. } => todo!("mouse wheel"),
            WinitEvent::MouseInput { .. } => todo!("mouse input"),
            WinitEvent::TouchpadPressure { .. } => todo!("touchpad pressure"),
            WinitEvent::AxisMotion { .. } => todo!("axis motion"),
            WinitEvent::Touch(_) => todo!("touch"),
            WinitEvent::ScaleFactorChanged { .. } => todo!("scale factor changed"),
            WinitEvent::ThemeChanged(_) => todo!("theme changed"),
        }
    }
    pub fn pull_events(&mut self) -> Vec<Event> {
        let mut events = self.events.clone();
        if self.quit {
            events.push(Event::ProgramTermination)
        }
        self.events.clear();
        return events;
    }
    pub fn quit_done(&mut self) -> bool {
        self.quit
    }
}
