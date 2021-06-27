use anyhow::Result;
use ash::vk;
use image::RgbaImage;
use nalgebra::{Matrix4, Vector2};
mod command_pool;
mod framebuffer;
mod render_core;
mod renderpass;
mod resource_pool;
use command_pool::CommandPool;
use framebuffer::{
    AttachableFramebuffer, AttachmentType, ColorBuffer, DepthBuffer, FrameBufferTarget,
    Framebuffer, TextureAttachment,
};
use generational_arena::{Arena, Index as ArenaIndex};
use pipeline::{GraphicsPipeline, PipelineType, ShaderDescription};
use render_core::Core;
use std::collections::HashMap;
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
const MAIN_SHADER: ShaderDescription = pipeline::PUSH_SHADER;
pub struct Backend {
    #[allow(dead_code)]
    window: winit::window::Window,
    vertex_buffers: Arena<VertexBufferAllocation>,
    index_buffers: Arena<IndexBufferAllocation>,
    textures: Arena<TextureAllocation>,
    framebuffer_arena: Arena<AttachableFramebuffer>,
    command_pool: CommandPool,
    resource_pool: ResourcePool,
    main_framebuffer: Framebuffer,
    renderpass: RenderPass,
    uniforms: HashMap<String, UniformAllocation>,
    main_graphics_pipeline: GraphicsPipeline,
    framebuffer_pipeline: GraphicsPipeline,
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
pub enum MeshTexture {
    RegularTexture(TextureID),
    Framebuffer(FramebufferID),
}
#[derive(Clone, Copy)]
pub struct MeshID {
    pub verticies: VertexBufferID,
    pub texture: MeshTexture,
    pub indicies: IndexBufferID,
}
impl MeshID {
    pub fn bind_texture(&mut self, tex: TextureID) {
        self.texture = MeshTexture::RegularTexture(tex);
    }
    pub fn bind_framebuffer(&mut self, fb: FramebufferID) {
        self.texture = MeshTexture::Framebuffer(fb);
    }
}
#[derive(Clone, Copy)]
pub struct FramebufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy)]
pub enum BoundFramebuffer {
    ScreenFramebuffer,
    UserFramebuffer(FramebufferID),
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
        let mut resource_pool = ResourcePool::new(&core, &MAIN_SHADER)?;
        let mut command_pool = CommandPool::new(&mut core);
        let texture_attachment = TextureAttachment::new(
            &mut core,
            &mut command_pool,
            &mut resource_pool,
            AttachmentType::Swapchain,
            create_info.default_size,
        )?;

        let format = core.surface_format.format;
        let mut main_graphics_pipeline = GraphicsPipeline::new(
            &mut core,
            &MAIN_SHADER,
            &MAIN_SHADER.vertex_buffer_desc,
            &resource_pool.get_descriptor_set_layouts(),
            &MAIN_SHADER
                .push_constants
                .into_iter()
                .map(|(k, v)| ((*k).to_string(), *v))
                .collect(),
            create_info.default_size.x,
            create_info.default_size.y,
            &texture_attachment.depth_buffer,
            format,
            PipelineType::Present,
        );
        let framebuffer_pipeline = GraphicsPipeline::new(
            &mut core,
            &MAIN_SHADER,
            &MAIN_SHADER.vertex_buffer_desc,
            &resource_pool.get_descriptor_set_layouts(),
            &MAIN_SHADER
                .push_constants
                .into_iter()
                .map(|(k, v)| ((*k).to_string(), *v))
                .collect(),
            create_info.default_size.x,
            create_info.default_size.y,
            &texture_attachment.depth_buffer,
            //todo allow custom framebuffer formats
            format,
            PipelineType::OffScreen,
        );

        let main_framebuffer = Framebuffer::new(
            &mut core,
            &mut main_graphics_pipeline,
            texture_attachment,
            create_info.default_size,
        )?;
        let renderpass = RenderPass::new(
            &mut core,
            &command_pool,
            &main_framebuffer.framebuffer_target,
        );
        let screen_dimensions = create_info.default_size;
        let uniforms = MAIN_SHADER
            .uniforms
            .into_iter()
            .map(|(name, desc)| {
                let data = vec![0u8; desc.size];
                (
                    name.to_string(),
                    resource_pool
                        .allocate_uniform(
                            &mut core,
                            main_framebuffer.num_swapchain_images() as u32,
                            data,
                            desc.descriptor_set_layout_binding,
                        )
                        .expect("failed to allocate uniform"),
                )
            })
            .collect();

        Ok(Self {
            window,
            core,
            resource_pool,
            command_pool,
            main_framebuffer,
            renderpass,
            main_graphics_pipeline,
            framebuffer_pipeline,
            screen_dimensions,
            uniforms,
            index_buffers: Arena::new(),
            vertex_buffers: Arena::new(),
            framebuffer_arena: Arena::new(),
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
    pub fn build_framebuffer(&mut self, resolution: Vector2<u32>) -> Result<FramebufferID> {
        Ok(FramebufferID {
            buffer_index: self.framebuffer_arena.insert(AttachableFramebuffer::new(
                &mut self.core,
                &mut self.command_pool,
                &mut self.framebuffer_pipeline,
                &mut self.resource_pool,
                resolution,
            )?),
        })
    }
    pub fn bind_framebuffer(&mut self, framebuffer_id: &BoundFramebuffer) -> Result<()> {
        let (pipeline, framebuffer) = match framebuffer_id {
            &BoundFramebuffer::ScreenFramebuffer => {
                (&mut self.main_graphics_pipeline, &self.main_framebuffer)
            }
            &BoundFramebuffer::UserFramebuffer(id) => (
                &mut self.framebuffer_pipeline,
                self.framebuffer_arena
                    .get(id.buffer_index)
                    .unwrap()
                    .get_framebuffer(),
            ),
        };
        unsafe {
            self.renderpass.end_renderpass(&mut self.core)?;
            self.renderpass.begin_renderpass(
                &mut self.core,
                pipeline,
                &framebuffer,
                self.screen_dimensions,
                ClearOp::ClearColor,
            )?;
        }
        Ok(())
    }
    pub fn draw_mesh(&mut self, view_matrix: Matrix4<f32>, mesh: &MeshID) -> Result<()> {
        let (pipeline, texture_descriptor_set) = match mesh.texture {
            MeshTexture::RegularTexture(texture) => (
                &mut self.main_graphics_pipeline,
                self.textures
                    .get(texture.buffer_index)
                    .unwrap()
                    .descriptor_set,
            ),
            MeshTexture::Framebuffer(fb) => (
                &mut self.framebuffer_pipeline,
                self.framebuffer_arena
                    .get(fb.buffer_index)
                    .unwrap()
                    .get_descriptor_set(self.renderpass.get_image_index(&mut self.core)?),
            ),
        };
        let render_mesh = RenderMesh {
            view_matrix,
            vertex_buffer: self
                .vertex_buffers
                .get(mesh.verticies.buffer_index)
                .unwrap(),
            index_buffer: self.index_buffers.get(mesh.indicies.buffer_index).unwrap(),
        };
        let mut descriptor_sets = self
            .uniforms
            .iter()
            .map(|(_name, uni)| uni.buffers.iter().map(|(_buff, _all, set)| *set))
            .flatten()
            .collect::<Vec<_>>();
        descriptor_sets.push(texture_descriptor_set);
        self.renderpass.draw_mesh(
            &mut self.core,
            pipeline,
            &self.main_framebuffer,
            &descriptor_sets,
            self.screen_dimensions,
            render_mesh,
        )
    }
    /// begins rendering of frame
    pub fn begin_render(&mut self) -> Result<()> {
        unsafe {
            self.renderpass.begin_frame(
                &mut self.core,
                &mut self.main_graphics_pipeline,
                &mut self.main_framebuffer,
                self.screen_dimensions,
            )
        }
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
            self.main_framebuffer
                .free(&mut self.core, &mut self.resource_pool)?;
            let texture_attachment = TextureAttachment::new(
                &mut self.core,
                &mut self.command_pool,
                &mut self.resource_pool,
                AttachmentType::Swapchain,
                new_size,
            )?;

            self.main_graphics_pipeline.free(&mut self.core);
            let format = self.core.surface_format.format;
            self.main_graphics_pipeline = GraphicsPipeline::new(
                &mut self.core,
                &MAIN_SHADER,
                &MAIN_SHADER.vertex_buffer_desc,
                &self.resource_pool.get_descriptor_set_layouts(),
                &MAIN_SHADER
                    .push_constants
                    .into_iter()
                    .map(|(k, v)| ((*k).to_string(), *v))
                    .collect(),
                new_size.x,
                new_size.y,
                &texture_attachment.depth_buffer,
                format,
                PipelineType::Present,
            );

            self.main_framebuffer = Framebuffer::new(
                &mut self.core,
                &mut self.main_graphics_pipeline,
                texture_attachment,
                new_size,
            )?;
            self.renderpass.free(&mut self.core);
            self.renderpass = RenderPass::new(
                &mut self.core,
                &self.command_pool,
                &self.main_framebuffer.framebuffer_target,
            );
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
            for (_idx, mut fb) in self.framebuffer_arena.drain() {
                fb.free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free");
            }
            self.renderpass.free(&mut self.core);
            self.main_graphics_pipeline.free(&mut self.core);
            self.framebuffer_pipeline.free(&mut self.core);
            self.main_framebuffer
                .free(&mut self.core, &mut self.resource_pool)
                .expect("failed to drop framebuffer");

            self.command_pool.free(&mut self.core);
            self.resource_pool
                .free(&mut self.core)
                .expect("failed to free");
            self.core.free();
        }
    }
}
