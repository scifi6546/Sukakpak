use super::prelude::{RenderingCtx, Transform};
use legion::*;
use std::{cell::RefCell, rc::Rc};
pub mod event;
mod text;
pub use event::EventCollector;
use event::{EventListner, MouseButtonEvent};
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3, Vector4},
    Context,
};
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
    pub fn build_listner(&self) -> EventListner {
        let upper_right = self.transform.mat() * Vector4::new(0.5, 0.5, 0.0, 1.0);
        let lower_left = self.transform.mat() * Vector4::new(-0.5, -0.5, 0.0, 1.0);
        EventListner::new(
            Vector2::new(upper_right.x, upper_right.y),
            Vector2::new(lower_left.x, lower_left.y),
        )
    }
    pub fn insert(
        transform: Transform,
        world: &mut World,
        default_texture: sukakpak::MeshTexture,
        hover_texture: sukakpak::MeshTexture,
        click_texture: sukakpak::MeshTexture,
        context: Rc<RefCell<Context>>,
    ) -> Result<()> {
        let square = GuiSquare::new(
            transform,
            default_texture,
            hover_texture,
            click_texture,
            context,
        )?;
        let listner = square.build_listner();
        world.push((square, listner));
        Ok(())
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
pub fn render_container(container: &VerticalContainer, #[resource] graphics: &mut RenderingCtx) {
    for (c, _event_collector) in container.items.iter() {
        let container_mat = container.container.transform.get_translate_mat();

        let mat = container.container.transform.get_translate_mat() * c.transform.mat();
        let mat = c.transform.mat() * container_mat;

        // let mat = c.transform.mat();

        graphics
            .0
            .borrow_mut()
            .draw_mesh(
                mat.as_slice()
                    .iter()
                    .map(|f| f.to_ne_bytes())
                    .flatten()
                    .collect(),
                &c.mesh,
            )
            .expect("failed to draw child");
    }
    graphics
        .0
        .borrow_mut()
        .draw_mesh(
            container.container.transform.to_bytes(),
            &container.container.mesh,
        )
        .expect("failed to draw mesh");
}
#[system(for_each)]
pub fn render_gui(square: &GuiSquare, #[resource] graphics: &mut RenderingCtx) {
    graphics
        .0
        .borrow_mut()
        .draw_mesh(square.transform.to_bytes(), &square.mesh)
        .expect("failed to draw mesh");
}
/// Describes which way to alighn elements in a container
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
    items: Vec<(GuiSquare, EventListner)>,
    container: GuiSquare,
}
impl VerticalContainer {
    pub fn new(
        mut items: Vec<GuiSquare>,
        style: VerticalContainerStyle,
        root_position: Vector3<f32>,
        context: Rc<RefCell<Context>>,
    ) -> Result<Self> {
        let height: f32 = items
            .iter()
            .map(|square| square.transform.get_scale().y + style.padding * 2.0)
            .sum();
        let width = items
            .iter()
            .map(|square| square.transform.get_scale().x + style.padding * 2.0)
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
            let z = 0.01;
            println!("y: {}", y);
            let item_height = item.transform.get_scale().y;
            item.transform = item.transform.clone().set_translation(Vector3::new(
                x,
                y + item_height / 2.0,
                -0.01,
            ));
            y += item.transform.get_scale().y + style.padding;
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
    pub fn build_listner(&self) -> EventListner {
        self.container.build_listner()
    }

    pub fn insert(
        items: Vec<GuiSquare>,
        style: VerticalContainerStyle,
        root_position: Vector3<f32>,
        world: &mut World,
        graphics_context: Rc<RefCell<Context>>,
    ) -> Result<()> {
        let container = VerticalContainer::new(items, style, root_position, graphics_context)?;
        let listner = container.build_listner();
        world.push((container, listner));
        Ok(())
    }
}
