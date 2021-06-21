use anyhow::Result;
use ash::vk;
use image::RgbaImage;
use nalgebra::{Matrix4, Vector2};
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
use pipeline::{GraphicsPipeline, ShaderDescription};
use present_image::PresentImage;
use render_core::Core;
mod pipeline;
use renderpass::{ClearOp, RenderMesh, RenderPass};
use resource_pool::{
    IndexBufferAllocation, ResourcePool, TextureAllocation, UniformAllocation,
    VertexBufferAllocation,
};

pub struct BackendCreateInfo {
    pub default_size: Vector2<u32>,
    pub name: String,
}
//layout of vertex
#[allow(non_camel_case_types)]
pub enum VertexLayout {
    XYZ_F32,    //xyz vector with floating point components
    XYZ_UV_F32, //xyz with uv
}
pub struct Backend {
    #[allow(dead_code)]
    window: winit::window::Window,
    vertex_buffers: Arena<VertexBufferAllocation>,
    index_buffers: Arena<IndexBufferAllocation>,
    textures: Arena<TextureAllocation>,
    command_pool: CommandPool,
    resource_pool: ResourcePool,
    present_image: PresentImage,
    depth_buffer: DepthBuffer,
    framebuffer: Framebuffer,
    renderpass: RenderPass,
    graphics_pipeline: GraphicsPipeline,
    screen_dimensions: Vector2<u32>,
    core: Core,
}
#[derive(Clone, Copy)]
pub struct VertexBufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy)]
pub struct IndexBufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy)]
pub struct TextureID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy)]
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
            &resource_pool.get_descriptor_set_layouts(),
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
        let renderpass = RenderPass::new(&mut core, &command_pool, &framebuffer);
        let screen_dimensions = create_info.default_size;
        Ok(Self {
            window,
            core,
            resource_pool,
            command_pool,
            present_image,
            framebuffer,
            depth_buffer,
            renderpass,
            graphics_pipeline,
            screen_dimensions,
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
    pub fn draw_mesh(&mut self, view_matrix: Matrix4<f32>, mesh: &MeshID) -> Result<()> {
        let render_mesh = RenderMesh {
            view_matrix,
            vertex_buffer: self
                .vertex_buffers
                .get(mesh.verticies.buffer_index)
                .unwrap(),
            index_buffer: self.index_buffers.get(mesh.indicies.buffer_index).unwrap(),
            texture: self.textures.get(mesh.texture.buffer_index).unwrap(),
        };
        //todo handle uniform descriptors
        self.renderpass.draw_mesh(
            &mut self.core,
            &self.graphics_pipeline,
            &self.framebuffer,
            &[render_mesh.texture.descriptor_set],
            self.screen_dimensions,
            render_mesh,
        )
    }
    /// begins rendering of frame
    pub fn begin_render(&mut self) -> Result<()> {
        self.renderpass.begin_renderpass(
            &mut self.core,
            &mut self.graphics_pipeline,
            &mut self.framebuffer,
            self.screen_dimensions,
            ClearOp::ClearColor,
        )
    }
    pub fn finish_render(&mut self) -> Result<()> {
        self.renderpass.submit_draw(&mut self.core)?;
        let r = self.renderpass.swap_framebuffer(&mut self.core);
        if let Err(r) = r {
            if r == vk::Result::ERROR_OUT_OF_DATE_KHR {
                let new_size = self.window.inner_size();
                let new_size = Vector2::new(new_size.width, new_size.height);
                println!("out of date khr");
                self.resize_renderer(new_size)?;
                Ok(())
            } else if r == vk::Result::SUBOPTIMAL_KHR {
                println!("sub optimal khr");
                Ok(())
            } else {
                Err(anyhow::anyhow!("Vk result: {}", r))
            }
        } else {
            Ok(())
        }
    }
    pub fn resize_renderer(&mut self, new_size: Vector2<u32>) -> Result<()> {
        if new_size == self.screen_dimensions {
            Ok(())
        } else {
            self.renderpass.wait_idle(&mut self.core);
            self.core.update_swapchain_resolution(new_size)?;
            self.depth_buffer
                .free(&mut self.core, &mut self.resource_pool)?;
            self.depth_buffer = DepthBuffer::new(
                &mut self.core,
                &mut self.command_pool,
                &mut self.resource_pool,
                new_size,
            )?;
            self.graphics_pipeline.free(&mut self.core);
            self.graphics_pipeline = GraphicsPipeline::new(
                &mut self.core,
                &pipeline::PUSH_SHADER,
                &pipeline::PUSH_SHADER.vertex_buffer_desc,
                &self.resource_pool.get_descriptor_set_layouts(),
                &pipeline::PUSH_SHADER
                    .push_constants
                    .into_iter()
                    .map(|(k, v)| ((*k).to_string(), *v))
                    .collect(),
                new_size.x,
                new_size.y,
                &self.depth_buffer,
            );
            self.present_image.free(&mut self.core);
            self.present_image = PresentImage::new(&mut self.core);
            self.framebuffer.free(&mut self.core);

            self.framebuffer = Framebuffer::new(
                &mut self.core,
                &mut self.present_image,
                &mut self.graphics_pipeline,
                &mut self.depth_buffer,
                new_size.x,
                new_size.y,
            );
            // self.resource_pool.free(&mut self.core)?;
            self.renderpass =
                RenderPass::new(&mut self.core, &self.command_pool, &self.framebuffer);
            self.screen_dimensions = new_size;
            Ok(())
        }
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.renderpass.wait_idle(&mut self.core);
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
            self.renderpass.free(&mut self.core);
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
