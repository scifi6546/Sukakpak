use super::Backend;
use nalgebra::Vector2;
use winit::event;
use winit::event::{
    ElementState as WState, MouseButton as WMouseButton, TouchPhase as WTouchPhase,
    WindowEvent as WinitEvent,
};

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
    ScrollStart {
        delta: ScrollDelta,
    },
    ScrollContinue {
        delta: ScrollDelta,
    },
    ScrollEnd {
        delta: ScrollDelta,
    },
    MouseMoved {
        ///Mouse position with y increasing as cursor goes up the window and x is increasing as
        ///the mouse moves to the right.
        position: Vector2<f32>,
        /// Normalized position where top right of the window is (1.0,1.0) and bottom left is (-1.0,-1.0)
        normalized: Vector2<f32>,
    },
    MouseDown {
        button: MouseButton,
    },
    MouseUp {
        button: MouseButton,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}
impl From<WMouseButton> for MouseButton {
    fn from(button: WMouseButton) -> Self {
        match button {
            WMouseButton::Left => Self::Left,
            WMouseButton::Right => Self::Right,
            WMouseButton::Middle => Self::Middle,
            WMouseButton::Other(b) => Self::Other(b),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollDelta {
    pub delta: Vector2<f32>,
}
impl ScrollDelta {
    pub fn x(&self) -> f32 {
        self.delta.x
    }
    pub fn y(&self) -> f32 {
        self.delta.y
    }
}
impl From<event::MouseScrollDelta> for ScrollDelta {
    fn from(delta: event::MouseScrollDelta) -> Self {
        match delta {
            event::MouseScrollDelta::LineDelta(x, y) => Self {
                delta: Vector2::new(x, y),
            },
            event::MouseScrollDelta::PixelDelta(pos) => Self {
                delta: Vector2::new(pos.x as f32, pos.y as f32),
            },
        }
    }
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
    pub fn push_event(&mut self, event: WinitEvent, backend: &Backend) {
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
            WinitEvent::CursorMoved { position, .. } => {
                let screen_size = backend.get_screen_size();
                self.events.push(Event::MouseMoved {
                    position: Vector2::new(
                        position.x as f32,
                        screen_size.y as f32 - position.y as f32,
                    ),
                    normalized: Vector2::new(
                        2.0 * (position.x as f32 / screen_size.x as f32 - 0.5),
                        2.0 * ((screen_size.y as f32 - position.y as f32) / screen_size.y as f32
                            - 0.5),
                    ),
                })
            }
            WinitEvent::CursorEntered { .. } => self.events.push(Event::CursorEnteredWindow),
            WinitEvent::CursorLeft { .. } => self.events.push(Event::CursorLeftWindow),
            WinitEvent::MouseWheel { delta, phase, .. } => match phase {
                WTouchPhase::Started => self.events.push(Event::ScrollStart {
                    delta: delta.into(),
                }),
                WTouchPhase::Moved => self.events.push(Event::ScrollContinue {
                    delta: delta.into(),
                }),
                WTouchPhase::Ended => self.events.push(Event::ScrollEnd {
                    delta: delta.into(),
                }),
                WTouchPhase::Cancelled => (),
            },
            WinitEvent::MouseInput { state, button, .. } => self.events.push(match state {
                WState::Pressed => Event::MouseDown {
                    button: button.into(),
                },
                WState::Released => Event::MouseUp {
                    button: button.into(),
                },
            }),
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
