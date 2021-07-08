use nalgebra::Vector2;
use winit::event::WindowEvent as WinitEvent;
#[derive(Clone, Debug)]
pub enum Event {
    ProgramTermination,
    WindowResized {
        new_size: Vector2<u32>,
    },
    WindowMoved {
        new_position: Vector2<i32>,
    },
    WindowGainedFocus,
    WindowLostFocus,
    CursorEnteredWindow,
    CursorLeftWindow,
    ControllerAxis {
        axis_id: u32,
        value: f32,
    },
    MouseMoved {
        ///Mouse position with y increasing as cursor goes down the window and x is increasing as
        ///the mouse moves to the right.
        position: Vector2<f32>,
    },
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
            WinitEvent::Resized(size) => self.events.push(Event::WindowResized {
                new_size: Vector2::new(size.width, size.height),
            }),
            WinitEvent::Moved(pos) => self.events.push(Event::WindowMoved {
                new_position: Vector2::new(pos.x, pos.y),
            }),

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
            WinitEvent::Focused(f) => {
                let e = if f {
                    Event::WindowGainedFocus
                } else {
                    Event::WindowLostFocus
                };
                self.events.push(e);
            }
            WinitEvent::KeyboardInput { .. } => todo!("keyboard input"),
            WinitEvent::ModifiersChanged(_) => todo!("Modifiers Changed"),
            WinitEvent::CursorMoved { position, .. } => self.events.push(Event::MouseMoved {
                position: Vector2::new(position.x as f32, position.y as f32),
            }),
            WinitEvent::CursorEntered { .. } => self.events.push(Event::CursorEnteredWindow),
            WinitEvent::CursorLeft { .. } => self.events.push(Event::CursorLeftWindow),
            WinitEvent::MouseWheel { .. } => todo!("mouse wheel"),
            WinitEvent::MouseInput { .. } => todo!("mouse input"),
            WinitEvent::TouchpadPressure { .. } => todo!("touchpad pressure"),
            WinitEvent::AxisMotion { axis, value, .. } => self.events.push(Event::ControllerAxis {
                axis_id: axis,
                value: value as f32,
            }),
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
