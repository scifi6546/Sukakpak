use super::prelude::{RenderingCtx, Transform};
use legion::*;
use std::{cell::RefCell, rc::Rc};
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector4},
    Context,
};
#[derive(Debug)]
pub struct GuiSquare {
    mesh: sukakpak::Mesh,
    default_texture: sukakpak::MeshTexture,
    hover_texture: sukakpak::MeshTexture,
    click_texture: sukakpak::MeshTexture,
}
impl GuiSquare {
    pub fn insert(
        transform: Transform,
        world: &mut World,
        context: Rc<RefCell<Context>>,
    ) -> Result<()> {
        let upper_right = transform.mat() * Vector4::new(0.5, 0.5, 0.0, 1.0);
        let lower_left = transform.mat() * Vector4::new(-0.5, -0.5, 0.0, 1.0);
        let default_texture = context.borrow_mut().build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([200, 20, 20, 200]),
        ))?;
        let hover_texture = context.borrow_mut().build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([180, 10, 10, 200]),
        ))?;
        let click_texture = context.borrow_mut().build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([100, 5, 5, 200]),
        ))?;

        let mesh = context.borrow_mut().build_mesh(
            sukakpak::MeshAsset {
                vertices: [
                    ((-0.5f32, 0.5, 0.0), (0.0, 0.0)),
                    ((0.5, 0.5, 0.0), (1.0, 0.0)),
                    ((-0.5, -0.5, 0.0), (0.0, 1.0)),
                    ((0.5, -0.5, 0.0), (1.0, 1.0)),
                ]
                .iter()
                .map(|((x, y, z), (u, v))| [x, y, z, u, v])
                .flatten()
                .map(|f| (*f).to_ne_bytes())
                .flatten()
                .collect(),
                indices: vec![0, 2, 1, 2, 3, 1],
                vertex_layout: sukakpak::VertexLayout {
                    components: vec![
                        sukakpak::VertexComponent::Vec3F32,
                        sukakpak::VertexComponent::Vec2F32,
                        sukakpak::VertexComponent::Vec3F32,
                    ],
                },
            },
            default_texture,
        );
        world.push((
            GuiSquare {
                mesh,
                default_texture,
                hover_texture,
                click_texture,
            },
            transform,
            EventListner::new(
                Vector2::new(upper_right.x, upper_right.y),
                Vector2::new(lower_left.x, lower_left.y),
            ),
        ));
        Ok(())
    }
}
#[system(for_each)]
pub fn react_events(square: &mut GuiSquare, event_listner: &EventListner) {
    if event_listner.left_mouse_down {
        square.mesh.bind_texture(square.click_texture);
    } else if event_listner.mouse_hovered {
        square.mesh.bind_texture(square.hover_texture);
    } else {
        square.mesh.bind_texture(square.default_texture);
    }
}
#[system(for_each)]
pub fn render_gui(
    square: &GuiSquare,
    transform: &Transform,
    #[resource] graphics: &mut RenderingCtx,
) {
    graphics
        .0
        .borrow_mut()
        .draw_mesh(transform.to_bytes(), &square.mesh)
        .expect("failed to draw mesh");
}
pub struct VerticalContainer {
    items: Vec<GuiSquare>,
    container: GuiSquare,
}
impl VerticalContainer {
    pub fn new(items: Vec<GuiSquare>) -> Self {
        todo!()
    }
}
/// Collects information for Gui events
pub struct EventCollector {
    last_mouse_pos: Vector2<f32>,
    right_mouse_down: bool,
    middle_mouse_down: bool,
    left_mouse_down: bool,
}
impl EventCollector {
    pub fn process_events(&mut self, events: &[sukakpak::Event]) {
        for event in events {
            match event {
                sukakpak::Event::MouseMoved { normalized, .. } => self.last_mouse_pos = *normalized,
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
                _ => {}
            }
        }
    }
}
impl Default for EventCollector {
    fn default() -> Self {
        Self {
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
        listner.right_mouse_down = collector.right_mouse_down;
        listner.middle_mouse_down = collector.middle_mouse_down;
        listner.left_mouse_down = collector.left_mouse_down;

        listner.mouse_hovered = true;
    }
}
/// Listner for mouse events. Coordinates are in regular cartesian with the upper right corner
/// being (1,1) and the lower left being (-1,-1)
pub struct EventListner {
    mouse_hovered: bool,
    #[allow(dead_code)]
    right_mouse_down: bool,
    #[allow(dead_code)]
    middle_mouse_down: bool,
    left_mouse_down: bool,
    upper_right_corner: Vector2<f32>,
    lower_left_corner: Vector2<f32>,
}
impl EventListner {
    /// resets events
    fn reset(&mut self) {
        self.mouse_hovered = false;
        self.right_mouse_down = false;
        self.middle_mouse_down = false;
        self.left_mouse_down = false;
    }
    /// checks if contains point in box
    pub fn contains_point(&self, point: Vector2<f32>) -> bool {
        (point.x < self.upper_right_corner.x && point.y < self.upper_right_corner.y)
            && (point.x > self.lower_left_corner.x && point.y > self.lower_left_corner.y)
    }
    pub fn new(upper_right_corner: Vector2<f32>, lower_left_corner: Vector2<f32>) -> Self {
        Self {
            mouse_hovered: false,
            upper_right_corner,
            lower_left_corner,
            right_mouse_down: false,
            middle_mouse_down: false,
            left_mouse_down: false,
        }
    }
}
