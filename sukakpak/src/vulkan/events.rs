use super::{
    super::{ControlFlow, WindowEvent},
    Event, MouseButton, ScrollDelta, SemanticKeyCode,
};
use nalgebra::Vector2;
use winit::event;
use winit::event::{
    ElementState as WState, Event as WinitEvent, MouseButton as WMouseButton,
    TouchPhase as WTouchPhase, WindowEvent as WinitWindowEvent,
};
use winit::event_loop::ControlFlow as WControllFlow;
use winit::keyboard::PhysicalKey;

impl From<WMouseButton> for MouseButton {
    fn from(button: WMouseButton) -> Self {
        match button {
            WMouseButton::Left => Self::Left,
            WMouseButton::Right => Self::Right,
            WMouseButton::Middle => Self::Middle,
            WMouseButton::Back => todo!("unsupported back"),
            WMouseButton::Forward => todo!("unsupported button, forward"),
            WMouseButton::Other(b) => Self::Other(b),
        }
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
struct WinitEventLoopAdaptorState {
    quit: bool,
    screen_size: Vector2<u32>,
}
impl WinitEventLoopAdaptorState {
    pub fn new(screen_size: Vector2<u32>) -> Self {
        Self {
            quit: false,
            screen_size,
        }
    }
}

pub struct WinitEventLoopAdaptor {
    event_loop: winit::event_loop::EventLoop<()>,
    screen_size: Vector2<u32>,
}
impl super::super::EventLoopTrait for WinitEventLoopAdaptor {
    fn new(screen_size: Vector2<u32>) -> Self {
        let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event loop");
        Self {
            event_loop,
            screen_size,
        }
    }
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, mut run_fn: F) {
        let mut state = WinitEventLoopAdaptorState::new(self.screen_size);
        self.event_loop
            .run(move |event, event_loop_window_target| {
                let flow: ControlFlow = match event {
                    WinitEvent::WindowEvent { event, .. } => {
                        if let Some(e) = Self::to_event(&mut state, event) {
                            let mut control_flow = ControlFlow::Continue;
                            run_fn(WindowEvent::Event(e), &mut control_flow);
                            control_flow
                        } else {
                            ControlFlow::Continue
                        }
                    }

                    _ => ControlFlow::Continue,
                };
                if flow == ControlFlow::Quit || state.quit {
                    event_loop_window_target.exit();
                }
            })
            .expect("failed to run event loop")
    }
}
impl WinitEventLoopAdaptor {
    pub fn event_loop(&self) -> &winit::event_loop::EventLoop<()> {
        &self.event_loop
    }
    fn to_event(state: &mut WinitEventLoopAdaptorState, event: WinitWindowEvent) -> Option<Event> {
        match event {
            WinitWindowEvent::Resized(size) => Some(Event::WindowResized {
                new_size: Vector2::new(size.width, size.height),
            }),
            WinitWindowEvent::Moved(pos) => Some(Event::WindowMoved {
                new_position: Vector2::new(pos.x, pos.y),
            }),

            WinitWindowEvent::CloseRequested => {
                state.quit = true;
                None
            }
            WinitWindowEvent::Destroyed => {
                state.quit = true;
                None
            }
            WinitWindowEvent::DroppedFile(_) => todo!("dropped file"),
            WinitWindowEvent::HoveredFile(_) => todo!("hover file"),
            WinitWindowEvent::HoveredFileCancelled => todo!("hovered file canceled"),

            WinitWindowEvent::Focused(focused) => match focused {
                true => Some(Event::WindowGainedFocus),
                false => Some(Event::WindowLostFocus),
            },
            WinitWindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let scan_code: u32 = match event.physical_key {
                    PhysicalKey::Code(_) => {
                        todo!("keycode")
                    }
                    PhysicalKey::Unidentified(_) => {
                        todo!("keycode")
                    }
                };
                let semantic_code = todo!("convert semantic code to code I can use");

                match event.state {
                    winit::event::ElementState::Pressed => Some(Event::KeyDown {
                        scan_code: todo!("scan code"),
                        semantic_code,
                    }),

                    winit::event::ElementState::Released => Some(Event::KeyUp {
                        scan_code: todo!("scan code"),
                        semantic_code,
                    }),
                }
            }
            WinitWindowEvent::ModifiersChanged(_) => None,
            WinitWindowEvent::CursorMoved { position, .. } => Some(Event::MouseMoved {
                position: Vector2::new(
                    position.x as f32,
                    state.screen_size.y as f32 - position.y as f32,
                ),
                normalized: Vector2::new(
                    2.0 * (position.x as f32 / state.screen_size.x as f32 - 0.5),
                    2.0 * ((state.screen_size.y as f32 - position.y as f32)
                        / state.screen_size.y as f32
                        - 0.5),
                ),
            }),
            WinitWindowEvent::CursorEntered { .. } => Some(Event::CursorEnteredWindow),
            WinitWindowEvent::CursorLeft { .. } => Some(Event::CursorLeftWindow),
            WinitWindowEvent::MouseWheel { delta, phase, .. } => match phase {
                WTouchPhase::Started => Some(Event::ScrollStart {
                    delta: delta.into(),
                }),
                WTouchPhase::Moved => Some(Event::ScrollContinue {
                    delta: delta.into(),
                }),
                WTouchPhase::Ended => Some(Event::ScrollEnd {
                    delta: delta.into(),
                }),
                WTouchPhase::Cancelled => None,
            },
            WinitWindowEvent::MouseInput { state, button, .. } => Some(match state {
                WState::Pressed => Event::MouseDown {
                    button: button.into(),
                },
                WState::Released => Event::MouseUp {
                    button: button.into(),
                },
            }),
            WinitWindowEvent::TouchpadPressure { .. } => todo!("touchpad pressure"),
            WinitWindowEvent::AxisMotion { axis, value, .. } => Some(Event::ControllerAxis {
                axis_id: axis,
                value: value as f32,
            }),
            WinitWindowEvent::Touch(_) => todo!("touch"),
            WinitWindowEvent::ScaleFactorChanged { .. } => todo!("scale factor changed"),
            WinitWindowEvent::ThemeChanged(_) => todo!("theme changed"),
            WinitWindowEvent::RedrawRequested => Some(Event::RedrawRequested),
            _ => todo!("other event: {:#?}", event),
        }
    }
}
