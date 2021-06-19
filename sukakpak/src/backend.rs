use anyhow::Result;
use image::RgbaImage;
use nalgebra::Vector2;
mod command_pool;
mod depth_buffer;
mod framebuffer;
mod present_image;
mod render_core;
mod renderpass;
mod resource_pool;
use command_pool::CommandPool;
use depth_buffer::DepthBuffer;
use framebuffer::Framebuffer;
use generational_arena::{Arena, Index as ArenaIndex};
use pipeline::{GraphicsPipeline, ShaderDescription, VertexBufferDesc};
use present_image::PresentImage;
use render_core::Core;
mod pipeline;
use resource_pool::{
    IndexBufferAllocation, ResourcePool, TextureAllocation, UniformAllocation,
    VertexBufferAllocation,
};

pub struct BackendCreateInfo {
    pub default_size: Vector2<u32>,
    pub name: String,
}
//layout of vertex
pub enum VertexLayout {
    XYZ_F32, //xyz vector with floating point components
}
pub struct Backend {
    window: winit::window::Window,
    vertex_buffers: Arena<VertexBufferAllocation>,
    index_buffers: Arena<IndexBufferAllocation>,
    textures: Arena<TextureAllocation>,
    command_pool: CommandPool,
    resource_pool: ResourcePool,
    present_image: PresentImage,
    depth_buffer: DepthBuffer,
    framebuffer: Framebuffer,
    graphics_pipeline: GraphicsPipeline,
    core: Core,
}
pub struct VertexBufferID {
    buffer_index: ArenaIndex,
}
pub struct IndexBufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy)]
pub struct TextureID {
    buffer_index: ArenaIndex,
}
pub struct MeshID {
    pub verticies: VertexBufferID,
    pub texture: TextureID,
    pub indicies: IndexBufferID,
}
impl Backend {
    pub fn new(
        create_info: BackendCreateInfo,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Result<Self> {
        let window = winit::window::WindowBuilder::new()
            .with_title(create_info.name.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                create_info.default_size.x,
                create_info.default_size.y,
            ))
            .build(&event_loop)?;
        let mut core = Core::new(&window, &create_info)?;
        let mut resource_pool = ResourcePool::new(&core, &pipeline::PUSH_SHADER)?;
        let mut command_pool = CommandPool::new(&mut core);

        let mut present_image = PresentImage::new(&mut core);
        let mut depth_buffer = DepthBuffer::new(
            &mut core,
            &mut command_pool,
            &mut resource_pool,
            create_info.default_size,
        )?;
        let mut graphics_pipeline = GraphicsPipeline::new(
            &mut core,
            &pipeline::PUSH_SHADER,
            &pipeline::PUSH_SHADER.vertex_buffer_desc,
            &resource_pool.get_descriptor_sets(),
            &pipeline::PUSH_SHADER
                .push_constants
                .into_iter()
                .map(|(k, v)| ((*k).to_string(), *v))
                .collect(),
            create_info.default_size.x,
            create_info.default_size.y,
            &depth_buffer,
        );

        let framebuffer = Framebuffer::new(
            &mut core,
            &mut present_image,
            &mut graphics_pipeline,
            &mut depth_buffer,
            create_info.default_size.x,
            create_info.default_size.y,
        );

        Ok(Self {
            window,
            core,
            resource_pool,
            command_pool,
            present_image,
            framebuffer,
            depth_buffer,
            graphics_pipeline,
            index_buffers: Arena::new(),
            vertex_buffers: Arena::new(),
            textures: Arena::new(),
        })
    }
    pub fn allocate_verticies(
        &mut self,
        mesh: Vec<u8>,
        vertex_layout: VertexLayout,
    ) -> Result<VertexBufferID> {
        Ok(VertexBufferID {
            buffer_index: self
                .vertex_buffers
                .insert(self.resource_pool.allocate_vertex_buffer(
                    &mut self.core,
                    mesh,
                    vertex_layout,
                )?),
        })
    }
    pub fn allocate_indicies(&mut self, indicies: Vec<u32>) -> Result<IndexBufferID> {
        Ok(IndexBufferID {
            buffer_index: self
                .index_buffers
                .insert(self.resource_pool.allocate_index_buffer(
                    &mut self.core,
                    &mut self.command_pool,
                    indicies,
                )?),
        })
    }
    pub fn allocate_texture(&mut self, texture: &RgbaImage) -> Result<TextureID> {
        Ok(TextureID {
            buffer_index: self.textures.insert(self.resource_pool.allocate_texture(
                &mut self.core,
                &mut self.command_pool,
                texture,
            )?),
        })
    }
    pub fn draw_mesh(&mut self, mesh: &MeshID) -> Result<()> {
        todo!()
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        unsafe {
            for (_idx, mesh) in self.vertex_buffers.drain() {
                mesh.free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free mesh");
            }
            for (_idx, buff) in self.index_buffers.drain() {
                buff.free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free buffer");
            }
            for (_idx, tex) in self.textures.drain() {
                tex.free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free textures");
            }
            self.graphics_pipeline.free(&mut self.core);
            self.framebuffer.free(&mut self.core);
            self.present_image.free(&mut self.core);
            self.depth_buffer
                .free(&mut self.core, &mut self.resource_pool)
                .expect("failed to free");
            self.command_pool.free(&mut self.core);
            self.resource_pool
                .free(&mut self.core)
                .expect("failed to free");
            self.core.free();
        }
    }
}
