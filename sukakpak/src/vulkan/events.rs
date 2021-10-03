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
    ControllerAxis {
        axis_id: u32,
        value: f32,
    },
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
    KeyDown {
        scan_code: u32,
        semantic_code: Option<SemanticKeyCode>,
    },
    ReceivedCharacter(char),
    KeyUp {
        scan_code: u32,
        semantic_code: Option<SemanticKeyCode>,
    },
}
#[derive(Clone, Debug)]
pub enum SemanticKeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}
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
pub struct WinitEventLoopAdaptor {
    event_loop: winit::event_loop::EventLoop<()>,
}
impl super::super::EventLoopTrait for WinitEventLoopAdaptor {
    fn new() -> Self {
        todo!("winit")
    }
    fn run<F: 'static + FnMut(super::super::WindowEvent, &mut super::super::ControlFlow)>(
        self,
        event: F,
    ) -> ! {
        todo!()
    }
}
impl WinitEventLoopAdaptor {
    pub fn event_loop(&self) -> &winit::event_loop::EventLoop<()> {
        &self.event_loop
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
            WinitEvent::ReceivedCharacter(c) => self.events.push(Event::ReceivedCharacter(c)),
            WinitEvent::Focused(f) => {
                let e = if f {
                    Event::WindowGainedFocus
                } else {
                    Event::WindowLostFocus
                };
                self.events.push(e);
            }
            WinitEvent::KeyboardInput { input, .. } => {
                let scan_code: u32 = input.scancode;
                let semantic_code = if let Some(code) = input.virtual_keycode {
                    Some(code.into())
                } else {
                    None
                };
                match input.state {
                    winit::event::ElementState::Pressed => self.events.push(Event::KeyDown {
                        scan_code,
                        semantic_code,
                    }),

                    winit::event::ElementState::Released => self.events.push(Event::KeyUp {
                        scan_code,
                        semantic_code,
                    }),
                }
            }
            WinitEvent::ModifiersChanged(_) => {}
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
        events
    }
    pub fn quit_done(&mut self) -> bool {
        self.quit
    }
}
