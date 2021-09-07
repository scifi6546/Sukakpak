use super::prelude::{Model, Transform};
use asset_manager::{AssetHandle, AssetManager};
use legion::*;
use std::sync::Mutex;
pub mod event;
mod text;
pub use event::EventCollector;
use event::EventListner;
use sukakpak::{
    anyhow::Result,
    image::{Rgba, RgbaImage},
    nalgebra::{Vector2, Vector3, Vector4},
    Context, DrawableTexture, Texture,
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
    fn render(
        &self,
        transform: Transform,
        graphics: &mut Context,
        model_manager: &AssetManager<Model>,
        texture_manager: &AssetManager<Texture>,
    );
    fn get_transform(&self) -> &Transform;
    fn set_transform(
        &mut self,
        transform: Transform,
        graphics: &mut Context,
        model_manager: &mut AssetManager<Model>,
        texture_manager: &mut AssetManager<Texture>,
    );
    fn build_listner(&self) -> EventListner;
}

#[derive(Debug)]
pub struct GuiSquare {
    mesh: AssetHandle<Model>,
    transform: Transform,
    default_texture: AssetHandle<Texture>,
    hover_texture: AssetHandle<Texture>,
    click_texture: AssetHandle<Texture>,
}
impl GuiSquare {
    pub fn new(
        transform: Transform,
        default_texture: AssetHandle<Texture>,
        hover_texture: AssetHandle<Texture>,
        click_texture: AssetHandle<Texture>,
        asset_manager: &mut AssetManager<Model>,
        texture_manager: &AssetManager<Texture>,
        context: &mut Context,
    ) -> Result<Self> {
        let raw_mesh = context.build_mesh(
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
            DrawableTexture::Texture(
                texture_manager
                    .get(&default_texture)
                    .expect("failed to get default texture"),
            ),
        )?;
        let mesh = asset_manager.insert(Model { mesh: raw_mesh });
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
    fn render(
        &self,
        transform: Transform,
        graphics: &mut Context,
        model_manager: &AssetManager<Model>,
        _texture_manager: &AssetManager<Texture>,
    ) {
        let mat = transform.mat() * self.transform.mat();
        graphics
            .draw_mesh(
                mat.iter().map(|f| f.to_ne_bytes()).flatten().collect(),
                &model_manager
                    .get(&self.mesh)
                    .expect("failed to get mesh")
                    .mesh,
            )
            .expect("failed to draw mesh");
    }
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
    fn set_transform(
        &mut self,
        transform: Transform,
        _graphics: &mut Context,
        _model_manager: &mut AssetManager<Model>,
        _texture_manager: &mut AssetManager<Texture>,
    ) {
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
pub fn react_events(
    square: &mut GuiSquare,
    event_listner: &EventListner,
    #[resource] context: &mut Context,
    #[resource] model_manager: &mut AssetManager<Model>,
    #[resource] texture_manager: &AssetManager<Texture>,
) {
    if event_listner.left_mouse_down.clicked() {
        context
            .bind_texture(
                &mut model_manager
                    .get_mut(&square.mesh)
                    .expect("failed to get")
                    .mesh,
                DrawableTexture::Texture(
                    texture_manager
                        .get(&square.click_texture)
                        .expect("failed to get texture"),
                ),
            )
            .expect("failed to bind");
    } else if event_listner.mouse_hovered.clicked() {
        context
            .bind_texture(
                &mut model_manager
                    .get_mut(&square.mesh)
                    .expect("failed to get model")
                    .mesh,
                DrawableTexture::Texture(texture_manager.get(&square.hover_texture).unwrap()),
            )
            .expect("failed to bind");
    } else {
        context.bind_texture(
            &mut model_manager.get_mut(&square.mesh).unwrap().mesh,
            DrawableTexture::Texture(texture_manager.get(&square.default_texture).unwrap()),
        );
    }
}
#[system(for_each)]
pub fn render_gui_component(
    component: &GuiComponent,
    #[resource] graphics: &mut Context,
    #[resource] model_manager: &AssetManager<Model>,
    #[resource] texture_manager: &AssetManager<Texture>,
) {
    component
        .item
        .lock()
        .expect("failed to get exclusive lock on gui item")
        .render(
            Transform::default(),
            graphics,
            model_manager,
            texture_manager,
        );
}
/// Describes which way to alighn elements in a container
pub struct TextLabel {
    text_mesh: AssetHandle<Model>,
    text_builder: TextBuilder,
    text_size: f32,
    /// Text that is displayed
    text: String,
    texture: AssetHandle<Texture>,
    /// Transform used for scaling text
    render_transform: Transform,
    /// Transform used for communicating size of mesh
    display_transform: Transform,
}
impl TextLabel {
    pub fn new(
        text: String,
        text_size: f32,
        // Determines size of box
        transform: Transform,
        context: &mut Context,
        model_manager: &mut AssetManager<Model>,
        texture_manager: &mut AssetManager<Texture>,
    ) -> Result<Self> {
        let mut text_builder = TextBuilder::default();
        let size = transform.get_scale().x;
        let max_line_width = (size * 2.0) / text_size;
        let text_info = TextInfo {
            text_size: [1, 1],
            max_line_width,
        };
        let (rgba_texture, bounding_box, mesh_asset) =
            text_builder.build_mesh(text_info, text.clone());
        let texture = context
            .build_texture(&rgba_texture)
            .expect("failed to text texture");
        let render_transform = {
            let scale_x = transform.get_scale().x;
            let mut scale = transform.get_scale();

            scale.x = scale_x / max_line_width;
            scale.y = scale_x / max_line_width;
            let middle = (bounding_box.max + bounding_box.min) / 2.0;
            let middle_translation =
                Vector3::new(-1.0 * middle.x * scale.x, -1.0 * middle.y * scale.x, 0.0);
            let translation = transform.get_translation();
            transform
                .clone()
                .set_scale(scale)
                .set_translation(translation)
                //    .translate(Vector3::new(scale_x / -2.0, 0.0, 0.0))
                .translate(middle_translation)
        };

        let display_scale = transform.get_scale();
        let display_transform = transform.set_scale(Vector3::new(
            display_scale.x,
            (bounding_box.max.y - bounding_box.min.y) * render_transform.get_scale().y,
            1.0,
        ));
        let text_mesh = model_manager.insert(Model {
            mesh: context.build_mesh(mesh_asset, DrawableTexture::Texture(&texture))?,
        });
        let texture = texture_manager.insert(texture);
        Ok(Self {
            text_mesh,
            texture,
            text,
            text_size,
            text_builder,
            render_transform,
            display_transform,
        })
    }
}
impl GuiItem for TextLabel {
    fn render(
        &self,
        transform: Transform,
        graphics: &mut Context,
        model_manager: &AssetManager<Model>,
        _texture_manager: &AssetManager<Texture>,
    ) {
        let mat = transform.mat() * self.render_transform.mat();
        graphics
            .draw_mesh(
                mat.iter().map(|f| f.to_ne_bytes()).flatten().collect(),
                &model_manager.get(&self.text_mesh).unwrap().mesh,
            )
            .expect("failed to render text");
    }
    fn get_transform(&self) -> &Transform {
        println!("getting transform: {}", self.display_transform);
        &self.display_transform
    }
    fn set_transform(
        &mut self,
        transform: Transform,
        graphics: &mut Context,
        model_manager: &mut AssetManager<Model>,
        texture_manager: &mut AssetManager<Texture>,
    ) {
        *self = Self::new(
            self.text.clone(),
            self.text_size,
            transform,
            graphics,
            model_manager,
            texture_manager,
        )
        .expect("failed to resize");
    }
    ///todo: figure out geometry properly
    fn build_listner(&self) -> EventListner {
        EventListner::new(Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerAlignment {
    Center,
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
        context: &mut Context,
        model_manager: &mut AssetManager<Model>,
        texture_manager: &mut AssetManager<Texture>,
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
        let mut y = height / -2.0;

        for item in items.iter_mut() {
            y += style.padding;
            let x = match style.alignment {
                ContainerAlignment::Center => 0.0,
            };
            println!("y: {}", y);
            let item_height = item.get_transform().get_scale().y;
            let item_transform = item.get_transform().clone().set_translation(Vector3::new(
                x,
                y + item_height / 2.0,
                -0.01,
            ));
            item.set_transform(item_transform, context, model_manager, texture_manager);

            y += item.get_transform().get_scale().y + style.padding;
        }
        let default_tex = texture_manager.insert(
            context
                .build_texture(&RgbaImage::from_pixel(
                    100,
                    100,
                    Rgba::from([0, 0, 100, 255]),
                ))
                .expect("failed to build default texture"),
        );
        let hover_tex = texture_manager.insert(
            context
                .build_texture(&RgbaImage::from_pixel(
                    100,
                    100,
                    Rgba::from([0, 0, 80, 255]),
                ))
                .expect("failed to build default texture"),
        );

        let click_tex = texture_manager.insert(
            context
                .build_texture(&RgbaImage::from_pixel(
                    100,
                    100,
                    Rgba::from([0, 0, 20, 255]),
                ))
                .expect("failed to build default texture"),
        );

        let container = GuiSquare::new(
            transform,
            default_tex,
            hover_tex,
            click_tex,
            model_manager,
            texture_manager,
            context,
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
    fn render(
        &self,
        transform: Transform,
        graphics: &mut Context,
        model_manager: &AssetManager<Model>,
        texture_manager: &AssetManager<Texture>,
    ) {
        for (c, _event_collector) in self.items.iter() {
            c.render(
                Transform::default().set_translation(
                    transform.get_translation() + self.get_transform().get_translation(),
                ),
                graphics,
                model_manager,
                texture_manager,
            );
        }
        graphics
            .draw_mesh(
                self.container.transform.to_bytes(),
                &model_manager.get(&self.container.mesh).unwrap().mesh,
            )
            .expect("failed to draw mesh");
    }
    fn get_transform(&self) -> &Transform {
        &self.container.transform
    }
    fn set_transform(
        &mut self,
        transform: Transform,
        graphics: &mut Context,
        model_manager: &mut AssetManager<Model>,
        texture_manager: &mut AssetManager<Texture>,
    ) {
        self.container
            .set_transform(transform, graphics, model_manager, texture_manager)
    }
    fn build_listner(&self) -> EventListner {
        self.container.build_listner()
    }
}
