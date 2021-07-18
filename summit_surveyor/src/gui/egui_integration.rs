use super::prelude::{
    na::{Vector2, Vector3, Vector4},
    Event, MeshAsset, RenderingCtx, Result, Texture as RGBATexture, VertexComponent, VertexLayout,
};
use egui::{
    math::{Pos2, Rect, Vec2},
    paint::tessellator::Triangles as EguiTris,
    PaintJobs, RawInput, Texture,
};
use log::info;
use std::sync::Arc;
/// Struct used to get state
///
pub struct EguiRawInputAdaptor {
    is_rightclick_down: bool,
    last_cursor_pos: Vector2<f32>,
    frame_scroll: f32,
}
impl EguiRawInputAdaptor {
    pub fn process_events(&mut self, events: &[Event], screen_size: Vector2<u32>) -> RawInput {
        self.frame_scroll = 0.0;
        for e in events.iter() {
            match e {
                Event::MouseDown { .. } => {
                    info!("mouse down");
                    self.is_rightclick_down = true;
                    info!("right click down: {}", self.is_rightclick_down);
                }
                Event::MouseUp { .. } => {
                    info!("mouse up ujjj");
                    info!("before??");
                    self.is_rightclick_down = false;
                    info!("right click down: {}", self.is_rightclick_down);
                }
                Event::MouseMoved { position, .. } => self.last_cursor_pos = *position,
                Event::ScrollContinue { delta, .. } => self.frame_scroll += delta.y(),
                _ => (),
            }
        }
        let mut input = RawInput::default();
        input.mouse_down = self.is_rightclick_down;
        input.mouse_pos = Some(Pos2::new(self.last_cursor_pos.x, self.last_cursor_pos.y));
        input.scroll_delta = Vec2::new(0.0, self.frame_scroll);
        input.screen_rect = Some(Rect {
            min: Pos2::new(0.0, 0.0),
            max: Pos2::new(screen_size.x as f32, screen_size.y as f32),
        });
        return input;
    }
}
impl Default for EguiRawInputAdaptor {
    fn default() -> Self {
        Self {
            is_rightclick_down: false,
            last_cursor_pos: Vector2::new(0.0, 0.0),
            frame_scroll: 0.0,
        }
    }
}
pub fn draw_egui(
    paint_jobs: &PaintJobs,
    texture: &Arc<Texture>,
    rendering_ctx: &RenderingCtx,
    screen_size: &Vector2<u32>,
) -> Result<()> {
    let pixels = texture
        .srgba_pixels()
        .map(|p| Vector4::new(p.r(), p.g(), p.b(), p.a()))
        .collect();
    let dimensions = Vector2::new(texture.width as u32, texture.height as u32);
    let texture = RGBATexture { pixels, dimensions };

    let render_texture = rendering_ctx
        .0
        .borrow_mut()
        .build_texture(&texture.into())?;
    let depth = -0.8;
    let mut vertices = vec![];
    let mut indices = vec![];
    for (_rect, triangles) in paint_jobs.iter() {
        let (mut v_out, mut i_out) = to_vertex(triangles, depth, screen_size);

        vertices.append(&mut v_out);
        indices.append(&mut i_out);
    }
    let mesh = rendering_ctx.0.borrow_mut().build_mesh(
        MeshAsset {
            vertices,
            indices,
            vertex_layout: VertexLayout {
                components: vec![
                    VertexComponent::Vec3F32,
                    VertexComponent::Vec2F32,
                    VertexComponent::Vec3F32,
                    VertexComponent::Vec4F32,
                ],
            },
        },
        render_texture,
    );
    rendering_ctx
        .0
        .borrow_mut()
        .draw_mesh(&[], &mesh)
        .expect("failed to draw");
    todo!("keep resurces for at least one frame");

    rendering_ctx.0.borrow_mut().delete_mesh(mesh);
    rendering_ctx.0.borrow_mut().delete_texture(render_texture);
    Ok(())
}

fn push_vec2(v: &Vector2<f32>, vec: &mut Vec<u8>) {
    let x = v.x.to_ne_bytes();
    for i in x {
        vec.push(i);
    }
    let y = v.y.to_ne_bytes();
    for i in y {
        vec.push(i);
    }
}
fn push_vec3(v: &Vector3<f32>, vec: &mut Vec<u8>) {
    let x = v.x.to_ne_bytes();
    for i in x {
        vec.push(i);
    }
    let y = v.y.to_ne_bytes();
    for i in y {
        vec.push(i);
    }
    let z = v.z.to_ne_bytes();
    for i in z {
        vec.push(i);
    }
}
fn push_vec4(v: &Vector4<f32>, vec: &mut Vec<u8>) {
    let x = v.x.to_ne_bytes();
    for i in x {
        vec.push(i);
    }
    let y = v.y.to_ne_bytes();
    for i in y {
        vec.push(i);
    }
    let z = v.z.to_ne_bytes();
    for i in z {
        vec.push(i);
    }
    let w = v.w.to_ne_bytes();
    for i in w {
        vec.push(i);
    }
}
fn to_vertex(triangles: &EguiTris, depth: f32, screen_size: &Vector2<u32>) -> (Vec<u8>, Vec<u32>) {
    let mut vertices = vec![];
    let screen_x = screen_size.x as f32 / 2.0;
    let screen_y = screen_size.y as f32 / 2.0;
    for vertex in triangles.vertices.iter() {
        let position = Vector3::new(
            vertex.pos.x / screen_x - 1.0,
            -1.0 * vertex.pos.y / screen_y + 1.0,
            depth,
        );

        let uv = Vector2::new(vertex.uv.x, vertex.uv.y);
        let color: egui::paint::Rgba = vertex.color.into();
        let color = Vector4::new(color.r(), color.g(), color.b(), color.a());

        push_vec3(&position, &mut vertices);
        push_vec2(&uv, &mut vertices);
        push_vec4(&color, &mut vertices);
    }
    (vertices, triangles.indices.clone())
}
