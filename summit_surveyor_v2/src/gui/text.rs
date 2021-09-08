use super::super::prelude::{AssetHandle, AssetManager};
use epaint::{
    text::{FontDefinitions, Fonts, TextStyle},
    TessellationOptions, Tessellator,
};
use std::collections::HashMap;
use sukakpak::{image::RgbaImage, nalgebra::Vector2, Context, MeshAsset, Texture};
struct Dimensions {
    width: u32,
    height: u32,
}
pub struct TextBuilder {
    font: Fonts,
    texture: Option<(AssetHandle<Texture>, Dimensions, BoundingBox)>,
    tesselator: Tessellator,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextInfo {
    pub text_size: [usize; 2],
    /// max width of line in points
    pub max_line_width: f32,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Vector2<f32>,
    pub max: Vector2<f32>,
}
impl std::fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\tmin: <{}, {}>\n\tmax: <{}, {}>\n}}",
            self.min.x, self.min.y, self.max.x, self.max.y
        )
    }
}
impl TextBuilder {
    pub fn build_mesh(
        &mut self,
        text_info: TextInfo,
        context: &mut Context,
        asset_manager: &mut AssetManager<Texture>,
        text: String,
    ) -> (AssetHandle<Texture>, BoundingBox, MeshAsset) {
        let mut mesh = epaint::Mesh::default();
        self.tesselator.tessellate_text(
            text_info.text_size,
            epaint::emath::Pos2 { x: 0.0, y: 0.0 },
            &self
                .font
                .layout_multiline(TextStyle::Body, text, text_info.max_line_width),
            epaint::color::Color32::BLACK,
            false,
            &mut mesh,
        );

        if self.texture.is_none() {
            let texture = self.font.texture();
            let image_data = texture
                .pixels
                .iter()
                .map(|v| [*v, *v, *v, *v])
                .flatten()
                .collect();
            let min_x = mesh
                .vertices
                .iter()
                .map(|v| v.pos.x)
                .reduce(f32::min)
                .unwrap_or(0.0);
            let min_y = mesh
                .vertices
                .iter()
                .map(|v| v.pos.y)
                .reduce(f32::min)
                .unwrap_or(0.0);
            let max_x = mesh
                .vertices
                .iter()
                .map(|v| v.pos.x)
                .reduce(f32::max)
                .unwrap_or(0.0);
            let max_y = mesh
                .vertices
                .iter()
                .map(|v| v.pos.y)
                .reduce(f32::max)
                .unwrap_or(0.0);
            let tex = asset_manager.insert(
                context
                    .build_texture(
                        &RgbaImage::from_vec(
                            texture.width as u32,
                            texture.height as u32,
                            image_data,
                        )
                        .unwrap(),
                    )
                    .expect("failed to build texture"),
            );
            self.texture = Some((
                tex,
                Dimensions {
                    height: texture.height as u32,
                    width: texture.width as u32,
                },
                BoundingBox {
                    min: Vector2::new(min_x, min_y),
                    max: Vector2::new(max_x, max_y),
                },
            ));
        };

        (
            self.texture.as_ref().unwrap().0.clone(),
            self.texture.as_ref().unwrap().2.clone(),
            MeshAsset {
                indices: mesh.indices,
                vertices: mesh
                    .vertices
                    .iter()
                    .map(|v| {
                        [
                            v.pos.x,
                            v.pos.y,
                            0.0,
                            v.uv.x / self.texture.as_ref().unwrap().1.width as f32,
                            v.uv.y / self.texture.as_ref().unwrap().1.height as f32,
                        ]
                    })
                    .flatten()
                    .map(|f| f.to_ne_bytes())
                    .flatten()
                    .collect(),
                vertex_layout: sukakpak::VertexLayout {
                    components: vec![
                        sukakpak::VertexComponent::Vec3F32,
                        sukakpak::VertexComponent::Vec2F32,
                        sukakpak::VertexComponent::Vec3F32,
                    ],
                },
            },
        )
    }
    fn new(font_size: FontSize) -> Self {
        Self {
            font: Fonts::from_definitions(font_size.0 as f32, FontDefinitions::default()),
            texture: None,
            tesselator: Tessellator::from_options(TessellationOptions::default()),
        }
    }
}
impl Default for TextBuilder {
    fn default() -> Self {
        Self {
            font: Fonts::from_definitions(10.0, FontDefinitions::default()),
            texture: None,
            tesselator: Tessellator::from_options(TessellationOptions::default()),
        }
    }
}
/// Denotes size of font in points
#[derive(PartialEq, Hash, Debug, Eq, Copy, Clone)]
pub struct FontSize(pub u32);
/// Stores text builders so that they are not reconstructed
pub struct TextBuilderContainer {
    builders: HashMap<FontSize, TextBuilder>,
}
impl Default for TextBuilderContainer {
    fn default() -> Self {
        Self {
            builders: HashMap::new(),
        }
    }
}
impl TextBuilderContainer {
    pub fn get(&mut self, font_size: FontSize) -> &TextBuilder {
        if self.builders.contains_key(&font_size) {
            self.builders.get(&font_size).unwrap()
        } else {
            self.builders
                .insert(font_size, TextBuilder::new(font_size.clone()));
            self.get(font_size)
        }
    }
    pub fn get_mut(&mut self, font_size: FontSize) -> &mut TextBuilder {
        if self.builders.contains_key(&font_size) {
            self.builders.get_mut(&font_size).unwrap()
        } else {
            self.builders
                .insert(font_size, TextBuilder::new(font_size.clone()));
            self.get_mut(font_size)
        }
    }
}
