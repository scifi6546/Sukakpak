use legion::*;
use std::collections::HashSet;
use std::time::Duration;
use sukakpak::nalgebra::Vector2;

/// Collects information for Gui events
pub struct EventCollector {
    pub keycodes_down: HashSet<u32>,
    pub mouse_delta_pos: Vector2<f32>,
    pub delta_time: Duration,
    pub mouse_scroll_delta: f32,
    pub last_mouse_pos: Vector2<f32>,
    pub right_mouse_down: bool,
    pub middle_mouse_down: bool,
    pub left_mouse_down: bool,
}
impl EventCollector {
    pub fn process_events(&mut self, delta_time: Duration, events: &[sukakpak::Event]) {
        self.delta_time = delta_time;
        self.mouse_delta_pos = Vector2::new(0.0, 0.0);
        self.mouse_scroll_delta = 0.0;
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
                sukakpak::Event::ScrollStart { delta } => self.mouse_scroll_delta += delta.delta.y,
                sukakpak::Event::ScrollContinue { delta } => {
                    self.mouse_scroll_delta += delta.delta.y
                }
                sukakpak::Event::ScrollEnd { delta } => self.mouse_scroll_delta += delta.delta.y,
                _ => {}
            }
        }
    }
    pub fn clear(&mut self) {}
}
impl Default for EventCollector {
    fn default() -> Self {
        Self {
            mouse_scroll_delta: 0.0,
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
pub fn send_events(listner: &mut EventListener, #[resource] collector: &EventCollector) {
    listner.receive_events(collector);
}
#[derive(Debug, Clone, PartialEq)]
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
        match *self {
            MouseButtonEvent::None => false,
            MouseButtonEvent::Clicked { .. } => true,
        }
    }
}
/// Listner for mouse events. Coordinates are in regular cartesian with the upper right corner
/// being (1,1) and the lower left being (-1,-1)
#[derive(Clone, Debug)]
pub struct EventListener {
    /// If mouse hovered over collider
    pub mouse_hovered: MouseButtonEvent,
    #[allow(dead_code)]
    /// If right mouse down over collider
    pub right_mouse_down: MouseButtonEvent,
    #[allow(dead_code)]
    /// If middle mouse down over collider
    pub middle_mouse_down: MouseButtonEvent,
    /// If left mouse down over collider
    pub left_mouse_down: MouseButtonEvent,
    pub upper_right_corner: Vector2<f32>,
    pub lower_left_corner: Vector2<f32>,
    pub sublistners: Vec<EventListener>,
}
impl EventListener {
    /// Receives events and sends them down to sublistners
    fn receive_events(&mut self, collector: &EventCollector) {
        self.reset();
        if self.contains_point(collector.last_mouse_pos) {
            if collector.right_mouse_down {
                self.right_mouse_down = MouseButtonEvent::Clicked {
                    position: collector.last_mouse_pos,
                };
            }
            if collector.middle_mouse_down {
                self.middle_mouse_down = MouseButtonEvent::Clicked {
                    position: collector.last_mouse_pos,
                };
            }
            if collector.left_mouse_down {
                self.left_mouse_down = MouseButtonEvent::Clicked {
                    position: collector.last_mouse_pos,
                };
            }
            self.mouse_hovered = MouseButtonEvent::Clicked {
                position: collector.last_mouse_pos,
            };
        }
        for listner in self.sublistners.iter_mut() {
            listner.receive_events(collector);
        }
    }
    pub fn add_sublistners(&mut self, mut sublistners: Vec<EventListener>) {
        self.sublistners.append(&mut sublistners);
    }
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
    #[allow(dead_code)]
    pub fn any_mouse_down(&self) -> bool {
        self.left_mouse_down != MouseButtonEvent::None
            || self.middle_mouse_down != MouseButtonEvent::None
            || self.right_mouse_down != MouseButtonEvent::None
    }
    #[allow(dead_code)]
    pub fn new(
        upper_right_corner: Vector2<f32>,
        lower_left_corner: Vector2<f32>,
        sublistners: Vec<EventListener>,
    ) -> Self {
        Self {
            mouse_hovered: MouseButtonEvent::None,
            upper_right_corner,
            lower_left_corner,
            right_mouse_down: MouseButtonEvent::None,
            middle_mouse_down: MouseButtonEvent::None,
            left_mouse_down: MouseButtonEvent::None,
            sublistners,
        }
    }
    /// If any mouse is down gets cursor position
    #[allow(dead_code)]
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
