mod mesh;
mod shader;
mod texture;

use mesh::Mesh;
use shader::ShaderModule;
use texture::Texture;

use anyhow::{bail, Result};
use generational_arena::{Arena, Index as ArenaIndex};
use image::RgbaImage;
use log::info;
use nalgebra::Vector2;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use std::{collections::HashMap, mem::size_of};

use super::super::{GenericBindable, GenericDrawableTexture, MeshAsset, VertexComponent};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawableTexture {
    Texture(TextureIndex),
    Framebuffer(Framebuffer),
}
#[derive(Debug)]
pub struct MeshIndex {
    index: ArenaIndex,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Framebuffer {}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureIndex {
    index: ArenaIndex,
}
pub struct Backend {
    quit: bool,
    context: WebGl2RenderingContext,
    shaders: HashMap<String, ShaderModule>,
    mesh_arena: Arena<Mesh>,
    texture_arena: Arena<Texture>,
    bound_shader: String,
}
impl Backend {
    pub fn new(backend: super::CreateBackend) -> Self {
        let canvas: HtmlCanvasElement = web_sys::window()
            .expect("failed to get window")
            .document()
            .expect("failed to get document")
            .get_element_by_id(&backend.create_info.window_id)
            .expect(&format!(
                "failed to get canvas with id: {}",
                backend.create_info.window_id
            ))
            .dyn_into()
            .expect("failed to convert to canvas");
        let mut context: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .expect("failed to get context")
            .expect("failed to get context")
            .dyn_into()
            .expect("failed to convert");
        let mut shaders = HashMap::new();
        let basic_shader =
            ShaderModule::basic_shader(&mut context).expect("failed to build basic shader");

        basic_shader
            .bind_shader(&mut context)
            .expect("failed to bind default shader");
        shaders.insert("basic".to_string(), basic_shader);

        let bound_shader = "basic".to_string();
        let mesh_arena = Arena::new();
        let texture_arena = Arena::new();
        Self {
            quit: false,
            context,
            shaders,
            bound_shader,
            mesh_arena,
            texture_arena,
        }
    }
    /// runs steps necessary for start of render
    pub fn begin_render(&mut self) -> Result<()> {
        self.context.clear_color(0.1, 0.1, 0.1, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        Ok(())
    }
    pub fn build_mesh(
        &mut self,
        mesh: MeshAsset,
        texture: GenericDrawableTexture<TextureIndex, Framebuffer>,
    ) -> Result<MeshIndex> {
        let buffer = self.context.create_buffer();
        if buffer.is_none() {
            bail!("failed to create buffer");
        }
        let buffer = buffer.unwrap();
        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        self.context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &mesh.vertices,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        let vao = self.context.create_vertex_array();
        if vao.is_none() {
            bail!("failed to create vertex array object")
        }
        let vao = vao.unwrap();
        self.context.bind_vertex_array(Some(&vao));
        let mut offset: usize = 0;
        let stride: usize = mesh.vertex_layout.components.iter().map(|v| v.size()).sum();
        for (location, vertex) in mesh.vertex_layout.components.iter().enumerate() {
            self.context.enable_vertex_attrib_array(location as u32);
            let normalized = false;
            self.context.vertex_attrib_pointer_with_i32(
                location as u32,
                vertex.num_components() as i32,
                match vertex {
                    VertexComponent::Vec1F32 => WebGl2RenderingContext::FLOAT,
                    VertexComponent::Vec2F32 => WebGl2RenderingContext::FLOAT,
                    VertexComponent::Vec3F32 => WebGl2RenderingContext::FLOAT,
                    VertexComponent::Vec4F32 => WebGl2RenderingContext::FLOAT,
                },
                normalized,
                stride as i32,
                offset as i32,
            );
            offset += vertex.size();
        }
        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        self.context.bind_vertex_array(None);
        let texture = match texture {
            GenericDrawableTexture::Texture(tex) => DrawableTexture::Texture(*tex),
            GenericDrawableTexture::Framebuffer(_) => todo!("framebuffer"),
        };

        let num_vertices = mesh.num_vertices();
        let mesh = Mesh {
            buffer,
            vao,
            texture,
            num_vertices,
        };
        let index = self.mesh_arena.insert(mesh);
        Ok(MeshIndex { index })
    }
    pub fn bind_texture(
        &mut self,
        _: &mut MeshIndex,
        _: GenericDrawableTexture<TextureIndex, Framebuffer>,
    ) -> Result<()> {
        todo!("bind texture")
    }
    pub fn build_texture(&mut self, image: &RgbaImage) -> Result<TextureIndex> {
        let gl_texture = self.context.create_texture();
        if gl_texture.is_none() {
            bail!("failed to create texture")
        }
        let gl_texture = gl_texture.unwrap();
        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0 + 0);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&gl_texture));
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        let mip_level = 0;
        let internal_format = WebGl2RenderingContext::RGBA as i32;
        //boarder of image must be zero
        let boarder = 0;
        let src_format = WebGl2RenderingContext::RGBA;
        let texel_type = WebGl2RenderingContext::UNSIGNED_BYTE;

        let result = self
            .context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                mip_level,
                internal_format,
                image.width() as i32,
                image.height() as i32,
                boarder,
                src_format,
                texel_type,
                image.as_raw(),
                0,
            );
        if result.is_err() {
            bail!("error in creating mesh")
        }
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        let index = self.texture_arena.insert(Texture {
            texture: gl_texture,
        });
        Ok(TextureIndex { index })
    }
    /// Very slow, todo: make finding uniform part of shader initilization
    pub fn draw_mesh(&mut self, push_data: Vec<u8>, mesh_index: &MeshIndex) -> Result<()> {
        let bound_shader = &self.shaders[&self.bound_shader];
        let num_uniforms = self
            .context
            .get_program_parameter(
                &bound_shader.program,
                WebGl2RenderingContext::ACTIVE_UNIFORMS,
            )
            .as_f64()
            .unwrap();
        info!("num uniforms: {}", num_uniforms);
        let mat4_uniform_attr = (0..num_uniforms as u32)
            .filter(|index| {
                let active_info = self
                    .context
                    .get_active_uniform(&bound_shader.program, *index)
                    .unwrap();
                active_info.type_() == WebGl2RenderingContext::FLOAT_MAT4
            })
            .next()
            .unwrap();
        info!("push attr: {}", mat4_uniform_attr);
        for index in 0..(num_uniforms as u32) {
            let active_info = self
                .context
                .get_active_uniform(&bound_shader.program, index)
                .unwrap();
            let type_num = active_info.type_();
            let type_str = match type_num {
                WebGl2RenderingContext::FLOAT_MAT4 => "matrix4".to_string(),
                WebGl2RenderingContext::SAMPLER_2D => "sampler 2d".to_string(),
                _ => format!("other({})", type_num),
            };
            info!(
                "{{\n\tname: {}\n\ttype: {}\n\tsize: {}\n}}",
                active_info.name(),
                type_str,
                active_info.size()
            );
            info!("{:#?}", active_info);
            let loc = self
                .context
                .get_uniform_location(&bound_shader.program, &active_info.name());
            info!("location: {:?} name: {}", loc, active_info.name());
        }
        let loc = (0..num_uniforms as u32)
            .filter(|index| {
                let active_info = self
                    .context
                    .get_active_uniform(&bound_shader.program, *index)
                    .unwrap();
                active_info
                    .name()
                    .contains(&bound_shader.shader.uniform_name)
            })
            .map(|index| {
                let active_info = self
                    .context
                    .get_active_uniform(&bound_shader.program, index)
                    .unwrap();
                let uniform_name = active_info.name();
                self.context
                    .get_uniform_location(&bound_shader.program, &uniform_name)
            })
            .next()
            .unwrap();
        info!("push loc: {:#?}", loc);

        info!("{}", bound_shader.shader.uniform_name);
        info!("{}", bound_shader.shader.vertex_shader);

        let float_arr = (0..16)
            .map(|i| {
                f32::from_ne_bytes([
                    push_data[0 + i * size_of::<f32>()],
                    push_data[1 + i * size_of::<f32>()],
                    push_data[2 + i * size_of::<f32>()],
                    push_data[3 + i * size_of::<f32>()],
                ])
            })
            .collect::<Vec<_>>();
        info!("float arr: {:#?}", float_arr);
        self.context
            .uniform_matrix4fv_with_f32_array(loc.as_ref(), false, &float_arr);
        let mesh = &self.mesh_arena[mesh_index.index];
        let texture = match mesh.texture {
            DrawableTexture::Texture(index) => self.texture_arena[index.index].texture.clone(),
            DrawableTexture::Framebuffer(_) => todo!("draw framebuffer surfaces"),
        };

        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        let texture_loc = self
            .context
            .get_uniform_location(&bound_shader.program, &bound_shader.shader.texture_name);
        info!("texture loc: {:#?}", texture_loc);
        self.context.uniform1i(texture_loc.as_ref(), 0);
        self.context.bind_vertex_array(Some(&mesh.vao));
        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&mesh.buffer));
        let offset = 0;
        self.context.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            offset,
            mesh.get_num_vertices() as i32,
        );
        self.context.bind_vertex_array(None);
        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        Ok(())
    }

    pub fn build_framebuffer(&mut self, _: Vector2<u32>) -> Result<Framebuffer> {
        todo!("build framebuffer")
    }
    pub fn bind_shader(&mut self, _: GenericBindable<Framebuffer>, _: &str) -> Result<()> {
        todo!("bind shader")
    }
    pub fn bind_framebuffer(&mut self, _: GenericBindable<Framebuffer>) -> Result<()> {
        todo!("bind framebuffer")
    }
    pub fn get_screen_size(&self) -> Vector2<u32> {
        todo!("get screen size")
    }
    pub fn load_shader(&mut self, _shader_text: &str, _name: &str) -> Result<()> {
        todo!("load shader")
    }
    pub fn quit(&mut self) {
        self.quit = true
    }
    pub fn did_quit(&self) -> bool {
        self.quit
    }
    pub fn check_state(&mut self) {}
}
