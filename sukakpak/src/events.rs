use nalgebra::Vector2;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
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
    RedrawRequested,
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
