mod mesh;
mod shader;
mod texture;
use super::{
    BackendTrait, ContextTrait, ControlFlow, CreateInfo, EventLoopTrait, GenericBindable,
    GenericDrawableTexture, MeshAsset, Timer, VertexComponent, WindowEvent,
};
use anyhow::{bail, Result};
use ass_wgl::Shader;
use generational_arena::{Arena, Index as ArenaIndex};
use image::RgbaImage;
use log::{info, Level};
use mesh::Mesh;
use nalgebra::Vector2;
use shader::ShaderModule;
use std::{collections::HashMap, mem::size_of, path::Path, time::Duration};
use texture::Texture;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject as VAO,
};
pub struct EventLoop {}
impl EventLoopTrait for EventLoop {
    fn new(_: Vector2<u32>) -> Self {
        console_log::init_with_level(Level::Debug);
        Self {}
    }
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, mut game_fn: F) -> ! {
        let mut flow = ControlFlow::Continue;
        loop {
            game_fn(WindowEvent::RunGameLogic, &mut flow);
            if flow == ControlFlow::Quit {
                panic!()
            }
        }
    }
}
pub struct Backend {
    create_info: CreateInfo,
}
impl BackendTrait for Backend {
    type EventLoop = EventLoop;
    fn new(create_info: CreateInfo, _: &Self::EventLoop) -> Self {
        Self { create_info }
    }
}
pub struct TimerContainer {
    /// time in ms
    time: f64,
}
impl Timer for TimerContainer {
    fn now() -> Self {
        let time = web_sys::window()
            .expect("failed to get window")
            .performance()
            .expect("failed to get performance")
            .now();
        Self { time }
    }
    fn elapsed(&self) -> Duration {
        let time = web_sys::window()
            .expect("failed to get window")
            .performance()
            .expect("failed to get performance")
            .now();
        let ms = time - self.time;
        Duration::from_micros((ms * 1000.0) as u64)
    }
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
/// For now only supporting uniforms with a 4x4 matrix
pub struct Context {
    quit: bool,
    context: WebGl2RenderingContext,
    shaders: HashMap<String, ShaderModule>,
    mesh_arena: Arena<Mesh>,
    texture_arena: Arena<Texture>,
    bound_shader: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawableTexture {
    Texture(TextureIndex),
    Framebuffer(Framebuffer),
}
///safe becuase web browsers do not have threads
unsafe impl Send for Context {}
unsafe impl Sync for Context {}
impl ContextTrait for Context {
    type Backend = Backend;
    type Mesh = MeshIndex;
    type Framebuffer = Framebuffer;
    type Texture = TextureIndex;
    type Timer = TimerContainer;
    fn new(backend: Self::Backend) -> Self {
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
    fn begin_render(&mut self) -> Result<()> {
        Ok(())
    }
    fn finish_render(&mut self) -> Result<()> {
        Ok(())
    }
    fn build_mesh(
        &mut self,
        mesh: MeshAsset,
        texture: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<Self::Mesh> {
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

        let mesh = Mesh {
            buffer,
            vao,
            texture,
        };
        let index = self.mesh_arena.insert(mesh);
        Ok(MeshIndex { index })
    }
    fn bind_texture(
        &mut self,
        _: &mut Self::Mesh,
        _: GenericDrawableTexture<Self::Texture, Self::Framebuffer>,
    ) -> Result<()> {
        todo!("bind texture")
    }
    fn build_texture(&mut self, image: &RgbaImage) -> Result<Self::Texture> {
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
    fn draw_mesh(&mut self, push_data: Vec<u8>, mesh_index: &Self::Mesh) -> Result<()> {
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

        todo!("draw mesh")
    }

    fn build_framebuffer(&mut self, _: Vector2<u32>) -> Result<Self::Framebuffer> {
        Ok(Framebuffer {})
    }
    fn bind_shader(&mut self, _: GenericBindable<Self::Framebuffer>, _: &str) -> Result<()> {
        Ok(())
    }
    fn bind_framebuffer(&mut self, _: GenericBindable<Self::Framebuffer>) -> Result<()> {
        Ok(())
    }
    fn get_screen_size(&self) -> Vector2<u32> {
        Vector2::new(100, 100)
    }
    fn load_shader(&mut self, _shader_text: &str, _name: &str) -> Result<()> {
        Ok(())
    }
    fn quit(&mut self) {
        self.quit = true
    }
    fn did_quit(&self) -> bool {
        self.quit
    }
    fn check_state(&mut self) {}
    fn clone(&self) -> Self {
        info!("cloning");
        let quit = self.quit.clone();
        Self {
            quit,
            context: self.context.clone(),
            shaders: self.shaders.clone(),
            bound_shader: self.bound_shader.clone(),
            mesh_arena: self.mesh_arena.clone(),
            texture_arena: self.texture_arena.clone(),
        }
    }
}
