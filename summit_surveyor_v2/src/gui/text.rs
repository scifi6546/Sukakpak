use epaint::{
    text::{FontDefinitions, Fonts, TextStyle},
    TessellationOptions, Tessellator,
};
use sukakpak::{image::RgbaImage, nalgebra::Vector2, MeshAsset};
pub struct TextBuilder {
    font: Fonts,
    tesselator: Tessellator,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextInfo {
    pub text_size: [usize; 2],
    /// max width of line in points
    pub max_line_width: f32,
}
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
        text: String,
    ) -> (RgbaImage, BoundingBox, MeshAsset) {
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

        (
            RgbaImage::from_vec(texture.width as u32, texture.height as u32, image_data).unwrap(),
            BoundingBox {
                min: Vector2::new(min_x, min_y),
                max: Vector2::new(max_x, max_y),
            },
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
                            v.uv.x / texture.width as f32,
                            v.uv.y / texture.height as f32,
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
}
impl Default for TextBuilder {
    fn default() -> Self {
        Self {
            font: Fonts::from_definitions(10.0, FontDefinitions::default()),
            tesselator: Tessellator::from_options(TessellationOptions::default()),
        }
    }
}
