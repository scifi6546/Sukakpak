use epaint::{
    text::{FontDefinitions, Fonts, TextStyle},
    TessellationOptions, Tessellator,
};
pub struct TextBuilder {
    font: Fonts,
    tesselator: Tessellator,
}
impl TextBuilder {
    pub fn new() -> Self {
        Self {
            font: Fonts::from_definitions(12.0, FontDefinitions::default()),
            tesselator: Tessellator::from_options(TessellationOptions::default()),
        }
    }
    pub fn build_shape(&mut self, text: String) -> epaint::Mesh {
        let mut mesh = epaint::Mesh::default();
        self.tesselator.tessellate_text(
            todo!("text size"),
            todo!("text position"),
            &self
                .font
                .layout_multiline(TextStyle::Body, text, todo!("max width")),
            todo!("color"),
            false,
            &mut mesh,
        );
        return mesh;
    }
}
