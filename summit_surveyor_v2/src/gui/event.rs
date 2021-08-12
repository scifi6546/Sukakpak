use legion::*;
use std::collections::HashSet;
use std::time::Duration;
use sukakpak::nalgebra::Vector2;

/// Collects information for Gui events
pub struct EventCollector {
    pub keycodes_down: HashSet<u32>,
    pub mouse_delta_pos: Vector2<f32>,
    pub delta_time: Duration,
    pub last_mouse_pos: Vector2<f32>,
    pub right_mouse_down: bool,
    pub middle_mouse_down: bool,
    pub left_mouse_down: bool,
}
impl EventCollector {
    pub fn process_events(&mut self, delta_time: Duration, events: &[sukakpak::Event]) {
        self.delta_time = delta_time;
        self.mouse_delta_pos = Vector2::new(0.0, 0.0);
        for event in events {
            match event {
                sukakpak::Event::MouseMoved { normalized, .. } => {
                    self.mouse_delta_pos = normalized - self.last_mouse_pos;
                    self.last_mouse_pos = *normalized;
                }
                sukakpak::Event::MouseDown { button } => match button {
                    sukakpak::MouseButton::Left => self.left_mouse_down = true,
                    sukakpak::MouseButton::Middle => self.middle_mouse_down = true,
                    sukakpak::MouseButton::Right => self.right_mouse_down = true,
                    sukakpak::MouseButton::Other(_) => {}
                },
                sukakpak::Event::MouseUp { button } => match button {
                    sukakpak::MouseButton::Left => self.left_mouse_down = false,
                    sukakpak::MouseButton::Middle => self.middle_mouse_down = false,
                    sukakpak::MouseButton::Right => self.right_mouse_down = false,
                    sukakpak::MouseButton::Other(_) => {}
                },
                sukakpak::Event::KeyDown { scan_code, .. } => {
                    println!("keydown scancode: {}", scan_code);
                    self.keycodes_down.insert(*scan_code);
                }
                sukakpak::Event::KeyUp { scan_code, .. } => {
                    println!("key up: {}", scan_code);
                    self.keycodes_down.remove(scan_code);
                }
                _ => {}
            }
        }
    }
    pub fn clear(&mut self) {}
}
impl Default for EventCollector {
    fn default() -> Self {
        Self {
            keycodes_down: HashSet::new(),
            delta_time: Default::default(),
            mouse_delta_pos: Vector2::new(0.0, 0.0),
            last_mouse_pos: Vector2::new(0.0, 0.0),
            right_mouse_down: false,
            middle_mouse_down: false,
            left_mouse_down: false,
        }
    }
}

#[system(for_each)]
pub fn send_events(listner: &mut EventListner, #[resource] collector: &EventCollector) {
    listner.reset();
    if listner.contains_point(collector.last_mouse_pos) {
        if collector.right_mouse_down {
            listner.right_mouse_down = MouseButtonEvent::Clicked {
                position: collector.last_mouse_pos,
            };
        }
        if collector.middle_mouse_down {
            listner.middle_mouse_down = MouseButtonEvent::Clicked {
                position: collector.last_mouse_pos,
            };
        }
        if collector.left_mouse_down {
            listner.left_mouse_down = MouseButtonEvent::Clicked {
                position: collector.last_mouse_pos,
            };
        }
        listner.mouse_hovered = MouseButtonEvent::Clicked {
            position: collector.last_mouse_pos,
        };
    }
}
#[derive(Debug, PartialEq)]
pub enum MouseButtonEvent {
    None,
    /// If clicked shows the position of the click relative to the center of the box
    Clicked {
        position: Vector2<f32>,
    },
}
impl MouseButtonEvent {
    /// returns true if click event is active
    pub fn clicked(&self) -> bool {
        match self {
            &MouseButtonEvent::None => false,
            &MouseButtonEvent::Clicked { .. } => true,
        }
    }
}
/// Listner for mouse events. Coordinates are in regular cartesian with the upper right corner
/// being (1,1) and the lower left being (-1,-1)
pub struct EventListner {
    pub mouse_hovered: MouseButtonEvent,
    #[allow(dead_code)]
    pub right_mouse_down: MouseButtonEvent,
    #[allow(dead_code)]
    pub middle_mouse_down: MouseButtonEvent,
    pub left_mouse_down: MouseButtonEvent,
    upper_right_corner: Vector2<f32>,
    lower_left_corner: Vector2<f32>,
}
impl EventListner {
    /// resets events
    fn reset(&mut self) {
        self.mouse_hovered = MouseButtonEvent::None;
        self.right_mouse_down = MouseButtonEvent::None;
        self.middle_mouse_down = MouseButtonEvent::None;
        self.left_mouse_down = MouseButtonEvent::None;
    }
    /// checks if contains point in box
    pub fn contains_point(&self, point: Vector2<f32>) -> bool {
        (point.x < self.upper_right_corner.x && point.y < self.upper_right_corner.y)
            && (point.x > self.lower_left_corner.x && point.y > self.lower_left_corner.y)
    }
    pub fn any_mouse_down(&self) -> bool {
        self.left_mouse_down != MouseButtonEvent::None
            || self.middle_mouse_down != MouseButtonEvent::None
            || self.right_mouse_down != MouseButtonEvent::None
    }
    pub fn new(upper_right_corner: Vector2<f32>, lower_left_corner: Vector2<f32>) -> Self {
        Self {
            mouse_hovered: MouseButtonEvent::None,
            upper_right_corner,
            lower_left_corner,
            right_mouse_down: MouseButtonEvent::None,
            middle_mouse_down: MouseButtonEvent::None,
            left_mouse_down: MouseButtonEvent::None,
        }
    }
    /// If any mouse is down gets cursor position
    pub fn get_mouse_pos(&self) -> Option<Vector2<f32>> {
        match self.right_mouse_down {
            MouseButtonEvent::Clicked { position } => Some(position),
            MouseButtonEvent::None => match self.middle_mouse_down {
                MouseButtonEvent::Clicked { position } => Some(position),
                MouseButtonEvent::None => match self.left_mouse_down {
                    MouseButtonEvent::Clicked { position } => Some(position),
                    MouseButtonEvent::None => None,
                },
            },
        }
    }
}
