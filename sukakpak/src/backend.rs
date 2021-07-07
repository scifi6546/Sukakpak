use anyhow::{anyhow, Result};
use ash::vk;
use image::RgbaImage;
use nalgebra::Vector2;
mod command_pool;
mod framebuffer;
mod render_core;
mod renderpass;
mod resource_pool;
use command_pool::CommandPool;
use framebuffer::{
    AttachableFramebuffer, AttachmentType, DepthBuffer, FrameBufferTarget, Framebuffer,
    TextureAttachment,
};
use generational_arena::{Arena, Index as ArenaIndex};
use pipeline::{alt_shader, push_shader, GraphicsPipeline, PipelineType, ShaderDescription};
use render_core::Core;
use thiserror::Error;
mod pipeline;
use renderpass::{ClearOp, RenderMesh, RenderPass};
use resource_pool::{
    DescriptorDesc, IndexBufferAllocation, ResourcePool, TextureAllocation, VertexBufferAllocation,
};
use std::collections::HashMap;

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
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Rendering to framebuffer {fb:?}")]
    RenderingBoundFramebuffer { fb: BoundFramebuffer },
}
pub struct Backend {
    #[allow(dead_code)]
    shaders: HashMap<String, ShaderDescription>,
    window: winit::window::Window,
    vertex_buffers: Arena<VertexBufferAllocation>,
    index_buffers: Arena<IndexBufferAllocation>,
    textures: Arena<TextureAllocation>,
    framebuffer_arena: Arena<AttachableFramebuffer>,
    command_pool: CommandPool,
    resource_pool: ResourcePool,
    main_framebuffer: Framebuffer,
    renderpass: RenderPass,
    bound_framebuffer: BoundFramebuffer,
    screen_dimensions: Vector2<u32>,
    main_shader: ShaderDescription,
    core: Core,
}
#[derive(Clone, Copy, Debug)]
pub struct VertexBufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy, Debug)]
pub struct IndexBufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy, Debug)]
pub struct TextureID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy, Debug)]
pub enum MeshTexture {
    RegularTexture(TextureID),
    Framebuffer(FramebufferID),
}
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FramebufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoundFramebuffer {
    ScreenFramebuffer,
    UserFramebuffer(FramebufferID),
}
impl From<&FramebufferID> for BoundFramebuffer {
    fn from(fb: &FramebufferID) -> Self {
        BoundFramebuffer::UserFramebuffer(*fb)
    }
}
impl Backend {
    pub fn new(
        create_info: BackendCreateInfo,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Result<Self> {
        let shaders = [
            ("push".to_string(), push_shader()),
            ("alt".to_string(), alt_shader()),
        ]
        .iter()
        .cloned()
        .collect();
        let main_shader = push_shader();
        let window = winit::window::WindowBuilder::new()
            .with_title(create_info.name.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                create_info.default_size.x,
                create_info.default_size.y,
            ))
            .build(&event_loop)?;
        let mut core = Core::new(&window, &create_info)?;
        let mut resource_pool = ResourcePool::new(&core, &main_shader)?;
        let mut command_pool = CommandPool::new(&mut core);
        let texture_attachment = TextureAttachment::new(
            &mut core,
            &mut command_pool,
            &mut resource_pool,
            AttachmentType::Swapchain,
            create_info.default_size,
        )?;

        let main_framebuffer = Framebuffer::new(
            &mut core,
            &main_shader,
            &resource_pool,
            texture_attachment,
            create_info.default_size,
            PipelineType::Present,
        )?;
        let renderpass = RenderPass::new(
            &mut core,
            &command_pool,
            &main_framebuffer.framebuffer_target,
        );
        let screen_dimensions = create_info.default_size;

        Ok(Self {
            window,
            main_shader,
            core,
            resource_pool,
            command_pool,
            main_framebuffer,
            renderpass,
            shaders,
            bound_framebuffer: BoundFramebuffer::ScreenFramebuffer,
            screen_dimensions,
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
                &mut self.resource_pool,
                &self.main_shader,
                resolution,
            )?),
        })
    }
    pub fn bind_framebuffer(&mut self, framebuffer_id: &BoundFramebuffer) -> Result<()> {
        let framebuffer = match framebuffer_id {
            &BoundFramebuffer::ScreenFramebuffer => (&self.main_framebuffer),
            &BoundFramebuffer::UserFramebuffer(id) => self
                .framebuffer_arena
                .get(id.buffer_index)
                .unwrap()
                .get_framebuffer(),
        };
        unsafe {
            self.renderpass.end_renderpass(&mut self.core)?;
            self.renderpass
                .begin_renderpass(&mut self.core, &framebuffer, ClearOp::ClearColor)?;
        }
        self.bound_framebuffer = *framebuffer_id;
        Ok(())
    }
    pub fn bind_shader(&mut self, framebuffer: &BoundFramebuffer, shader: &str) -> Result<()> {
        let shader = self.shaders.get(shader).unwrap();
        let framebuffer = match framebuffer {
            BoundFramebuffer::ScreenFramebuffer => &mut self.main_framebuffer,
            BoundFramebuffer::UserFramebuffer(id) => {
                &mut self
                    .framebuffer_arena
                    .get_mut(id.buffer_index)
                    .unwrap()
                    .framebuffer
            }
        };
        framebuffer.rebuild_framebuffer(&mut self.core, &self.resource_pool, shader)?;
        Ok(())
    }

    pub fn draw_mesh(&mut self, push: &[u8], mesh: &MeshID) -> Result<()> {
        let texture_descriptor_set = match mesh.texture {
            MeshTexture::RegularTexture(texture) => {
                self.textures
                    .get(texture.buffer_index)
                    .unwrap()
                    .descriptor_set
            }
            MeshTexture::Framebuffer(fb) => {
                if BoundFramebuffer::UserFramebuffer(fb) == self.bound_framebuffer {
                    return Err(anyhow!(
                        "{}",
                        RenderError::RenderingBoundFramebuffer {
                            fb: self.bound_framebuffer
                        }
                    ));
                } else {
                    self.framebuffer_arena
                        .get(fb.buffer_index)
                        .unwrap()
                        .get_descriptor_set(self.renderpass.get_image_index(&mut self.core)?)
                }
            }
        };
        let render_mesh = RenderMesh {
            push,
            vertex_buffer: self
                .vertex_buffers
                .get(mesh.verticies.buffer_index)
                .unwrap(),
            index_buffer: self.index_buffers.get(mesh.indicies.buffer_index).unwrap(),
        };
        let descriptor_set = [texture_descriptor_set];
        self.renderpass.draw_mesh(
            &mut self.core,
            match self.bound_framebuffer {
                BoundFramebuffer::ScreenFramebuffer => &self.main_framebuffer,
                BoundFramebuffer::UserFramebuffer(fb) => {
                    &self
                        .framebuffer_arena
                        .get(fb.buffer_index)
                        .unwrap()
                        .framebuffer
                }
            },
            &descriptor_set,
            self.screen_dimensions,
            render_mesh,
        )
    }
    /// begins rendering of frame
    pub fn begin_render(&mut self) -> Result<()> {
        unsafe {
            self.renderpass.begin_frame(
                &mut self.core,
                match self.bound_framebuffer {
                    BoundFramebuffer::ScreenFramebuffer => &self.main_framebuffer,
                    BoundFramebuffer::UserFramebuffer(fb) => {
                        &self
                            .framebuffer_arena
                            .get(fb.buffer_index)
                            .unwrap()
                            .framebuffer
                    }
                },
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

            self.main_framebuffer = Framebuffer::new(
                &mut self.core,
                &self.main_shader,
                &mut self.resource_pool,
                texture_attachment,
                new_size,
                PipelineType::Present,
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
    pub fn get_screen_size(&self) -> Vector2<u32> {
        self.screen_dimensions
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
