use super::{
    super::{ControlFlow, WindowEvent},
    Event, MouseButton, SemanticKeyCode,ScrollDelta
};
use nalgebra::Vector2;
use winit::event;
use winit::event::{
    ElementState as WState, Event as WinitEvent, MouseButton as WMouseButton,
    TouchPhase as WTouchPhase, WindowEvent as WinitWindowEvent,
};
use winit::event_loop::ControlFlow as WControllFlow;

impl From<winit::event::VirtualKeyCode> for SemanticKeyCode {
    fn from(code: winit::event::VirtualKeyCode) -> Self {
        use winit::event::VirtualKeyCode as VC;
        match code {
            VC::Key1 => SemanticKeyCode::Key1,
            VC::Key2 => SemanticKeyCode::Key2,
            VC::Key3 => SemanticKeyCode::Key3,
            VC::Key4 => SemanticKeyCode::Key4,
            VC::Key5 => SemanticKeyCode::Key5,
            VC::Key6 => SemanticKeyCode::Key6,
            VC::Key7 => SemanticKeyCode::Key7,
            VC::Key8 => SemanticKeyCode::Key8,
            VC::Key9 => SemanticKeyCode::Key9,
            VC::Key0 => SemanticKeyCode::Key0,

            VC::A => SemanticKeyCode::A,
            VC::B => SemanticKeyCode::B,
            VC::C => SemanticKeyCode::C,
            VC::D => SemanticKeyCode::D,
            VC::E => SemanticKeyCode::E,
            VC::F => SemanticKeyCode::F,
            VC::G => SemanticKeyCode::G,
            VC::H => SemanticKeyCode::H,
            VC::I => SemanticKeyCode::I,
            VC::J => SemanticKeyCode::J,
            VC::K => SemanticKeyCode::K,
            VC::L => SemanticKeyCode::L,
            VC::M => SemanticKeyCode::M,
            VC::N => SemanticKeyCode::N,
            VC::O => SemanticKeyCode::O,
            VC::P => SemanticKeyCode::P,
            VC::Q => SemanticKeyCode::Q,
            VC::R => SemanticKeyCode::R,
            VC::S => SemanticKeyCode::S,
            VC::T => SemanticKeyCode::T,
            VC::U => SemanticKeyCode::U,
            VC::V => SemanticKeyCode::V,
            VC::W => SemanticKeyCode::W,
            VC::X => SemanticKeyCode::X,
            VC::Y => SemanticKeyCode::Y,
            VC::Z => SemanticKeyCode::Z,
            VC::Escape => SemanticKeyCode::Escape,

            VC::F1 => SemanticKeyCode::F1,
            VC::F2 => SemanticKeyCode::F2,
            VC::F3 => SemanticKeyCode::F3,
            VC::F4 => SemanticKeyCode::F4,
            VC::F5 => SemanticKeyCode::F5,
            VC::F6 => SemanticKeyCode::F6,
            VC::F7 => SemanticKeyCode::F7,
            VC::F8 => SemanticKeyCode::F8,
            VC::F9 => SemanticKeyCode::F9,
            VC::F10 => SemanticKeyCode::F10,
            VC::F11 => SemanticKeyCode::F11,
            VC::F12 => SemanticKeyCode::F12,
            VC::F13 => SemanticKeyCode::F13,
            VC::F14 => SemanticKeyCode::F14,
            VC::F15 => SemanticKeyCode::F15,
            VC::F16 => SemanticKeyCode::F16,
            VC::F17 => SemanticKeyCode::F17,
            VC::F18 => SemanticKeyCode::F18,
            VC::F19 => SemanticKeyCode::F19,
            VC::F20 => SemanticKeyCode::F20,
            VC::F21 => SemanticKeyCode::F21,
            VC::F22 => SemanticKeyCode::F22,
            VC::F23 => SemanticKeyCode::F23,
            VC::F24 => SemanticKeyCode::F24,
            VC::Snapshot => SemanticKeyCode::Snapshot,
            VC::Scroll => SemanticKeyCode::Scroll,
            VC::Pause => SemanticKeyCode::Pause,

            VC::Insert => SemanticKeyCode::Insert,
            VC::Home => SemanticKeyCode::Home,
            VC::Delete => SemanticKeyCode::Delete,
            VC::End => SemanticKeyCode::End,
            VC::PageDown => SemanticKeyCode::PageDown,
            VC::PageUp => SemanticKeyCode::PageUp,
            VC::Left => SemanticKeyCode::Left,
            VC::Up => SemanticKeyCode::Up,
            VC::Right => SemanticKeyCode::Right,
            VC::Down => SemanticKeyCode::Down,
            VC::Back => SemanticKeyCode::Back,
            VC::Return => SemanticKeyCode::Return,
            VC::Space => SemanticKeyCode::Space,
            VC::Compose => SemanticKeyCode::Compose,
            VC::Caret => SemanticKeyCode::Caret,
            VC::Numlock => SemanticKeyCode::Numlock,
            VC::Numpad0 => SemanticKeyCode::Numpad0,
            VC::Numpad1 => SemanticKeyCode::Numpad1,
            VC::Numpad2 => SemanticKeyCode::Numpad2,
            VC::Numpad3 => SemanticKeyCode::Numpad3,
            VC::Numpad4 => SemanticKeyCode::Numpad4,
            VC::Numpad5 => SemanticKeyCode::Numpad5,
            VC::Numpad6 => SemanticKeyCode::Numpad6,
            VC::Numpad7 => SemanticKeyCode::Numpad7,
            VC::Numpad8 => SemanticKeyCode::Numpad8,
            VC::Numpad9 => SemanticKeyCode::Numpad9,
            VC::NumpadAdd => SemanticKeyCode::NumpadAdd,
            VC::NumpadDivide => SemanticKeyCode::NumpadDivide,
            VC::NumpadDecimal => SemanticKeyCode::NumpadDecimal,
            VC::NumpadComma => SemanticKeyCode::Comma,
            VC::NumpadEnter => SemanticKeyCode::NumpadEnter,
            VC::NumpadEquals => SemanticKeyCode::NumpadEquals,
            VC::NumpadMultiply => SemanticKeyCode::NumpadMultiply,
            VC::NumpadSubtract => SemanticKeyCode::NumpadSubtract,

            VC::AbntC1 => SemanticKeyCode::AbntC1,
            VC::AbntC2 => SemanticKeyCode::AbntC2,
            VC::Apostrophe => SemanticKeyCode::Apostrophe,
            VC::Apps => SemanticKeyCode::Apps,
            VC::Asterisk => SemanticKeyCode::Asterisk,
            VC::At => SemanticKeyCode::At,
            VC::Ax => SemanticKeyCode::Ax,
            VC::Backslash => SemanticKeyCode::Backslash,
            VC::Calculator => SemanticKeyCode::Calculator,
            VC::Capital => SemanticKeyCode::Capital,
            VC::Colon => SemanticKeyCode::Colon,
            VC::Comma => SemanticKeyCode::Comma,
            VC::Convert => SemanticKeyCode::Convert,
            VC::Equals => SemanticKeyCode::Equals,
            VC::Grave => SemanticKeyCode::Grave,
            VC::Kana => SemanticKeyCode::Kana,
            VC::Kanji => SemanticKeyCode::Kanji,
            VC::LAlt => SemanticKeyCode::LAlt,
            VC::LBracket => SemanticKeyCode::LBracket,
            VC::LControl => SemanticKeyCode::LControl,
            VC::LShift => SemanticKeyCode::LShift,
            VC::LWin => SemanticKeyCode::LWin,
            VC::Mail => SemanticKeyCode::Mail,
            VC::MediaSelect => SemanticKeyCode::MediaSelect,
            VC::MediaStop => SemanticKeyCode::MediaStop,
            VC::Minus => SemanticKeyCode::Minus,
            VC::Mute => SemanticKeyCode::Mute,
            VC::MyComputer => SemanticKeyCode::MyComputer,
            VC::NavigateForward => SemanticKeyCode::NavigateForward,
            VC::NavigateBackward => SemanticKeyCode::NavigateBackward,
            VC::NextTrack => SemanticKeyCode::NextTrack,
            VC::NoConvert => SemanticKeyCode::NoConvert,
            VC::OEM102 => SemanticKeyCode::OEM102,
            VC::Period => SemanticKeyCode::Period,
            VC::PlayPause => SemanticKeyCode::PlayPause,
            VC::Plus => SemanticKeyCode::Plus,
            VC::Power => SemanticKeyCode::Power,
            VC::PrevTrack => SemanticKeyCode::PrevTrack,
            VC::RAlt => SemanticKeyCode::RAlt,
            VC::RBracket => SemanticKeyCode::RBracket,
            VC::RControl => SemanticKeyCode::RControl,
            VC::RShift => SemanticKeyCode::RShift,
            VC::RWin => SemanticKeyCode::RWin,
            VC::Semicolon => SemanticKeyCode::Semicolon,
            VC::Slash => SemanticKeyCode::Slash,
            VC::Sleep => SemanticKeyCode::Sleep,
            VC::Stop => SemanticKeyCode::Stop,
            VC::Sysrq => SemanticKeyCode::Sysrq,
            VC::Tab => SemanticKeyCode::Tab,
            VC::Underline => SemanticKeyCode::Underline,
            VC::Unlabeled => SemanticKeyCode::Unlabeled,
            VC::VolumeDown => SemanticKeyCode::VolumeDown,
            VC::VolumeUp => SemanticKeyCode::VolumeUp,
            VC::Wake => SemanticKeyCode::Wake,
            VC::WebBack => SemanticKeyCode::WebBack,
            VC::WebFavorites => SemanticKeyCode::WebFavorites,
            VC::WebForward => SemanticKeyCode::WebForward,
            VC::WebHome => SemanticKeyCode::WebHome,
            VC::WebRefresh => SemanticKeyCode::WebRefresh,
            VC::WebSearch => SemanticKeyCode::WebSearch,
            VC::WebStop => SemanticKeyCode::WebStop,
            VC::Yen => SemanticKeyCode::Yen,
            VC::Copy => SemanticKeyCode::Copy,
            VC::Paste => SemanticKeyCode::Paste,
            VC::Cut => SemanticKeyCode::Cut,
        }
    }
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
        let event_loop = winit::event_loop::EventLoop::new();
        Self {
            event_loop,
            screen_size,
        }
    }
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, mut run_fn: F) -> ! {
        let mut state = WinitEventLoopAdaptorState::new(self.screen_size);
        self.event_loop.run(move |event, _, control_flow| {
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
                WinitEvent::MainEventsCleared => {
                    let mut control_flow = ControlFlow::Continue;
                    run_fn(WindowEvent::RunGameLogic, &mut control_flow);
                    control_flow
                }
                _ => ControlFlow::Continue,
            };
            if flow == ControlFlow::Quit || state.quit {
                *control_flow = WControllFlow::Exit
            }
        })
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
            WinitWindowEvent::ReceivedCharacter(c) => Some(Event::ReceivedCharacter(c)),
            WinitWindowEvent::Focused(focused) => match focused {
                true => Some(Event::WindowGainedFocus),
                false => Some(Event::WindowLostFocus),
            },
            WinitWindowEvent::KeyboardInput { input, .. } => {
                let scan_code: u32 = input.scancode;
                let semantic_code = if let Some(code) = input.virtual_keycode {
                    Some(code.into())
                } else {
                    None
                };
                match input.state {
                    winit::event::ElementState::Pressed => Some(Event::KeyDown {
                        scan_code,
                        semantic_code,
                    }),

                    winit::event::ElementState::Released => Some(Event::KeyUp {
                        scan_code,
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
        }
    }
}
