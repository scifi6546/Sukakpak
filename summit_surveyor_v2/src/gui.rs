use super::prelude::{RenderingCtx, Transform};
use legion::*;
use std::{cell::RefCell, rc::Rc, sync::Mutex};
pub mod event;
mod text;
pub use event::EventCollector;
use event::EventListner;
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3, Vector4},
    Context,
};
use text::{TextBuilder, TextInfo};
pub struct GuiComponent {
    pub item: Mutex<Box<dyn GuiItem>>,
}
impl GuiComponent {
    pub fn insert(item: Box<dyn GuiItem>, world: &mut World) -> Result<()> {
        world.push((
            Self {
                item: Mutex::new(item),
            },
            0u8,
        ));
        Ok(())
    }
}
pub trait GuiItem: Send {
    /// Renders the gui the transformation is applied to the box in the order of
    /// `transform.mat()*self.transform.mat()`
    fn render(&self, transform: Transform, graphics: &mut RenderingCtx);
    fn get_transform(&self) -> &Transform;
    fn set_transform(&mut self, transform: Transform);
    fn build_listner(&self) -> EventListner;
}

#[derive(Debug)]
pub struct GuiSquare {
    mesh: sukakpak::Mesh,
    transform: Transform,
    default_texture: sukakpak::MeshTexture,
    hover_texture: sukakpak::MeshTexture,
    click_texture: sukakpak::MeshTexture,
}
impl GuiSquare {
    pub fn new(
        transform: Transform,
        default_texture: sukakpak::MeshTexture,
        hover_texture: sukakpak::MeshTexture,
        click_texture: sukakpak::MeshTexture,
        context: Rc<RefCell<Context>>,
    ) -> Result<Self> {
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
        Ok(GuiSquare {
            mesh,
            default_texture,
            hover_texture,
            click_texture,
            transform,
        })
    }
}
impl GuiItem for GuiSquare {
    fn render(&self, transform: Transform, graphics: &mut RenderingCtx) {
        let mat = transform.mat() * self.transform.mat();
        graphics
            .0
            .borrow_mut()
            .draw_mesh(
                mat.iter().map(|f| f.to_ne_bytes()).flatten().collect(),
                &self.mesh,
            )
            .expect("failed to draw mesh");
    }
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform
    }
    fn build_listner(&self) -> EventListner {
        let upper_right = self.transform.mat() * Vector4::new(0.5, 0.5, 0.0, 1.0);
        let lower_left = self.transform.mat() * Vector4::new(-0.5, -0.5, 0.0, 1.0);
        EventListner::new(
            Vector2::new(upper_right.x, upper_right.y),
            Vector2::new(lower_left.x, lower_left.y),
        )
    }
}
#[system(for_each)]
pub fn react_events(square: &mut GuiSquare, event_listner: &EventListner) {
    if event_listner.left_mouse_down.clicked() {
        square.mesh.bind_texture(square.click_texture);
    } else if event_listner.mouse_hovered.clicked() {
        square.mesh.bind_texture(square.hover_texture);
    } else {
        square.mesh.bind_texture(square.default_texture);
    }
}
#[system(for_each)]
pub fn render_gui_component(component: &GuiComponent, #[resource] graphics: &mut RenderingCtx) {
    component
        .item
        .lock()
        .expect("failed to get exclusive lock on gui item")
        .render(Transform::default(), graphics);
}

/// Describes which way to alighn elements in a container
pub struct TextLabel {
    text_mesh: sukakpak::Mesh,
    text_builder: TextBuilder,
    texture: sukakpak::MeshTexture,
    transform: Transform,
}
impl TextLabel {
    pub fn new(text: String, transform: Transform, context: Rc<RefCell<Context>>) -> Self {
        let size = transform.get_scale().x;
        let mut text_builder = TextBuilder::default();
        let (rgba_texture, mesh_asset) = text_builder.build_mesh(
            TextInfo {
                text_size: [1, 1],
                max_line_width: 2.0 / size,
            },
            text,
        );
        println!(
            "dimensions: ({}, {})",
            rgba_texture.width(),
            rgba_texture.height()
        );
        let texture = context
            .borrow_mut()
            .build_texture(&rgba_texture)
            .expect("failed to text texture");
        let text_mesh = context.borrow_mut().build_mesh(mesh_asset, texture);
        Self {
            text_mesh,
            texture,
            text_builder,
            transform,
        }
    }
}
impl GuiItem for TextLabel {
    fn render(&self, transform: Transform, graphics: &mut RenderingCtx) {
        let mat = transform.mat() * self.transform.mat();
        graphics
            .0
            .borrow_mut()
            .draw_mesh(
                mat.iter().map(|f| f.to_ne_bytes()).flatten().collect(),
                &self.text_mesh,
            )
            .expect("failed to render text");
    }
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform
    }
    ///todo: figure out geometry properly
    fn build_listner(&self) -> EventListner {
        EventListner::new(Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerAlignment {
    Center,
    Left,
    Right,
}
#[derive(Debug, Clone, PartialEq)]
pub struct VerticalContainerStyle {
    /// padding inbetween elements
    pub padding: f32,
    pub alignment: ContainerAlignment,
}
/// Contains Vertical components of components
pub struct VerticalContainer {
    items: Vec<(Box<dyn GuiItem>, EventListner)>,
    container: GuiSquare,
}
impl VerticalContainer {
    pub fn new(
        mut items: Vec<Box<dyn GuiItem>>,
        style: VerticalContainerStyle,
        root_position: Vector3<f32>,
        context: Rc<RefCell<Context>>,
    ) -> Result<Self> {
        let height: f32 = items
            .iter()
            .map(|square| square.get_transform().get_scale().y + style.padding * 2.0)
            .sum();
        let width = items
            .iter()
            .map(|square| square.get_transform().get_scale().x + style.padding * 2.0)
            .fold(0.0, |acc, x| if acc > x { acc } else { x });
        let transform = Transform::default()
            .translate(root_position)
            .set_scale(Vector3::new(width, height, 1.0));
        println!("height: {} ", height);
        let mut y = height / -2.0;

        for item in items.iter_mut() {
            y += style.padding;
            let x = match style.alignment {
                ContainerAlignment::Left => todo!(),
                ContainerAlignment::Center => 0.0,
                ContainerAlignment::Right => todo!(),
            };
            println!("y: {}", y);
            let item_height = item.get_transform().get_scale().y;
            let item_transform = item.get_transform().clone().set_translation(Vector3::new(
                x,
                y + item_height / 2.0,
                -0.01,
            ));
            item.set_transform(item_transform);

            y += item.get_transform().get_scale().y + style.padding;
        }
        let default_tex = context
            .borrow_mut()
            .build_texture(&RgbaImage::from_pixel(
                100,
                100,
                Rgba::from([0, 0, 100, 255]),
            ))
            .expect("failed to build default texture");
        let hover_tex = context
            .borrow_mut()
            .build_texture(&RgbaImage::from_pixel(
                100,
                100,
                Rgba::from([0, 0, 80, 255]),
            ))
            .expect("failed to build default texture");

        let click_tex = context
            .borrow_mut()
            .build_texture(&RgbaImage::from_pixel(
                100,
                100,
                Rgba::from([0, 0, 20, 255]),
            ))
            .expect("failed to build default texture");

        let container = GuiSquare::new(
            transform,
            default_tex,
            hover_tex,
            click_tex,
            context.clone(),
        )?;
        Ok(Self {
            container,
            items: items
                .drain(..)
                .map(|square| {
                    let listner = square.build_listner();
                    (square, listner)
                })
                .collect(),
        })
    }
}
impl GuiItem for VerticalContainer {
    fn render(&self, transform: Transform, graphics: &mut RenderingCtx) {
        for (c, _event_collector) in self.items.iter() {
            c.render(
                Transform::default().set_translation(
                    transform.get_translation() + self.get_transform().get_translation(),
                ),
                graphics,
            );
        }
        graphics
            .0
            .borrow_mut()
            .draw_mesh(self.container.transform.to_bytes(), &self.container.mesh)
            .expect("failed to draw mesh");
    }
    fn get_transform(&self) -> &Transform {
        &self.container.transform
    }
    fn set_transform(&mut self, transform: Transform) {
        self.container.set_transform(transform)
    }
    fn build_listner(&self) -> EventListner {
        self.container.build_listner()
    }
}
