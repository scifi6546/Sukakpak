use anyhow::{anyhow, Result};
use ash::vk;
use ass_lib::asm_spv::load_from_fs;
use image::RgbaImage;
use nalgebra::Vector2;
use thiserror::Error;
mod command_pool;
mod framebuffer;
mod ref_counter;
mod render_core;
mod renderpass;
mod resource_pool;
mod vertex_layout;
use command_pool::CommandPool;
use framebuffer::{
    AttachableFramebuffer, AttachmentType, DepthBuffer, FrameBufferTarget, Framebuffer,
    TextureAttachment,
};
use generational_arena::{Arena, Index as ArenaIndex};
use pipeline::{alt_shader, push_shader, GraphicsPipeline, PipelineType, ShaderDescription};
use ref_counter::{RefCounter, RefrenceStatus};
use render_core::Core;
pub use vertex_layout::{VertexComponent, VertexLayout};
mod pipeline;
use renderpass::{ClearOp, RenderMesh, RenderMeshIds, RenderPass, ResourceId};
use resource_pool::{
    DescriptorDesc, IndexBufferAllocation, ResourcePool, TextureAllocation, VertexBufferAllocation,
};
use std::collections::HashSet;
use std::{collections::HashMap, path::Path};

pub struct BackendCreateInfo {
    pub default_size: Vector2<u32>,
    pub name: String,
}
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Rendering to framebuffer {fb:?}")]
    RenderingBoundFramebuffer { fb: BoundFramebuffer },
    #[error("Shader: {shader:} not found")]
    ShaderNotFound { shader: String },
}
pub struct Backend {
    #[allow(dead_code)]
    shaders: HashMap<String, ShaderDescription>,
    window: winit::window::Window,
    vertex_buffers: Arena<VertexBufferAllocation>,
    index_buffers: Arena<IndexBufferAllocation>,
    textures: Arena<RefCounter<TextureAllocation>>,
    to_free_textures: HashSet<MeshTexture>,
    framebuffer_arena: Arena<RefCounter<AttachableFramebuffer>>,
    command_pool: CommandPool,
    resource_pool: ResourcePool,
    main_framebuffer: Framebuffer,
    renderpass: RenderPass,
    bound_framebuffer: BoundFramebuffer,
    screen_dimensions: Vector2<u32>,
    main_shader: ShaderDescription,
    core: Core,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexBufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IndexBufferID {
    buffer_index: ArenaIndex,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureID {
    buffer_index: ArenaIndex,
}
/// Enum allowig both framebuffers and textures to be bound to mesh
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeshTexture {
    RegularTexture(TextureID),
    Framebuffer(FramebufferID),
}
#[derive(Clone, Copy, Debug)]
pub struct MeshID {
    vertices: VertexBufferID,
    texture: MeshTexture,
    indices: IndexBufferID,
}
impl MeshID {
    pub fn bind_texture(&mut self, tex: MeshTexture) {
        self.texture = tex;
    }
    pub fn bind_framebuffer(&mut self, fb: FramebufferID) {
        self.texture = MeshTexture::Framebuffer(fb);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
            .build(event_loop)?;
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
            to_free_textures: HashSet::new(),
        })
    }
    pub fn build_mesh(
        &mut self,
        verticies: Vec<u8>,
        vertex_layout: VertexLayout,
        indicies: Vec<u32>,
        texture: MeshTexture,
    ) -> Result<MeshID> {
        match texture {
            MeshTexture::RegularTexture(id) => self
                .textures
                .get_mut(id.buffer_index)
                .unwrap()
                .incr_refrence(),
            MeshTexture::Framebuffer(id) => self
                .framebuffer_arena
                .get_mut(id.buffer_index)
                .unwrap()
                .incr_refrence(),
        };
        Ok(MeshID {
            vertices: self.allocate_verticies(verticies, vertex_layout)?,
            indices: self.allocate_indicies(indicies)?,
            texture,
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
            buffer_index: self.textures.insert(RefCounter::new(
                self.resource_pool.allocate_texture(
                    &mut self.core,
                    &mut self.command_pool,
                    texture,
                )?,
                0,
            )),
        })
    }
    /// Lazily frees textures once the texture is no longer in use
    pub fn free_texture(&mut self, tex: MeshTexture) -> Result<()> {
        self.to_free_textures.insert(tex);
        Ok(())
    }
    /// Scans resources and frees all resources that need to be freed
    pub fn collect_garbage(&mut self) -> Result<()> {
        for tex in self.to_free_textures.iter() {
            match tex {
                MeshTexture::RegularTexture(id) => {
                    let num_refrences = self.textures.get(id.buffer_index).unwrap().refrences();
                    if num_refrences == 0
                        && !self
                            .renderpass
                            .is_resource_used(&ResourceId::UserTexture(id.buffer_index))
                    {
                        self.textures
                            .remove(id.buffer_index)
                            .unwrap()
                            .drain()
                            .free(&mut self.core, &mut self.resource_pool)?;
                    }
                }
                MeshTexture::Framebuffer(id) => {
                    let num_refrences = self
                        .framebuffer_arena
                        .get(id.buffer_index)
                        .unwrap()
                        .refrences();
                    if num_refrences == 0
                        && !self
                            .renderpass
                            .is_resource_used(&ResourceId::Framebuffer(id.buffer_index))
                    {
                        self.framebuffer_arena
                            .remove(id.buffer_index)
                            .unwrap()
                            .drain()
                            .free(&mut self.core, &mut self.resource_pool)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn build_framebuffer(&mut self, resolution: Vector2<u32>) -> Result<FramebufferID> {
        Ok(FramebufferID {
            buffer_index: self.framebuffer_arena.insert(RefCounter::new(
                AttachableFramebuffer::new(
                    &mut self.core,
                    &mut self.command_pool,
                    &mut self.resource_pool,
                    &self.main_shader,
                    resolution,
                )?,
                0,
            )),
        })
    }
    pub fn bind_framebuffer(&mut self, framebuffer_id: &BoundFramebuffer) -> Result<()> {
        let framebuffer = match *framebuffer_id {
            BoundFramebuffer::ScreenFramebuffer => (&self.main_framebuffer),
            BoundFramebuffer::UserFramebuffer(id) => self
                .framebuffer_arena
                .get(id.buffer_index)
                .unwrap()
                .get()
                .get_framebuffer(),
        };
        unsafe {
            self.renderpass.end_renderpass(&mut self.core)?;
            self.renderpass
                .begin_renderpass(&mut self.core, framebuffer, ClearOp::ClearColor)?;
        }
        self.bound_framebuffer = *framebuffer_id;
        Ok(())
    }
    pub fn bind_shader(&mut self, framebuffer: &BoundFramebuffer, shader: &str) -> Result<()> {
        let shader = if let Some(s) = self.shaders.get(shader) {
            s
        } else {
            return Err(anyhow!(
                "{}",
                RenderError::ShaderNotFound {
                    shader: shader.to_string()
                }
            ));
        };
        let framebuffer = match framebuffer {
            BoundFramebuffer::ScreenFramebuffer => &mut self.main_framebuffer,
            BoundFramebuffer::UserFramebuffer(id) => {
                &mut self
                    .framebuffer_arena
                    .get_mut(id.buffer_index)
                    .unwrap()
                    .get_mut()
                    .framebuffer
            }
        };
        framebuffer.rebuild_framebuffer(&mut self.core, &self.resource_pool, shader)?;
        Ok(())
    }
    /// Frees mesh data, can be called at any time as freeing waits untill data is unused by
    /// renderpasses
    pub fn free_mesh(&mut self, mesh: &MeshID) -> Result<()> {
        match mesh.texture {
            MeshTexture::RegularTexture(texture) => {
                self.textures
                    .get_mut(texture.buffer_index)
                    .unwrap()
                    .decr_refrence();
            }
            MeshTexture::Framebuffer(framebuffer) => {
                self.framebuffer_arena
                    .get_mut(framebuffer.buffer_index)
                    .unwrap()
                    .decr_refrence();
            }
        }
        let ids = RenderMeshIds {
            index_buffer_id: mesh.indices.buffer_index,
            vertex_buffer_id: mesh.vertices.buffer_index,
            texture_id: match mesh.texture {
                MeshTexture::RegularTexture(texture) => {
                    renderpass::TextureId::UserTexture(texture.buffer_index)
                }
                MeshTexture::Framebuffer(texture) => {
                    renderpass::TextureId::Framebuffer(texture.buffer_index)
                }
            },
        };
        self.renderpass.free_mesh(ids);
        Ok(())
    }

    pub fn draw_mesh(&mut self, push: Vec<u8>, mesh: &MeshID) -> Result<()> {
        let texture_descriptor_set = match mesh.texture {
            MeshTexture::RegularTexture(texture) => {
                self.textures
                    .get(texture.buffer_index)
                    .unwrap()
                    .get()
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
                        .get_mut(fb.buffer_index)
                        .unwrap()
                        .get_mut()
                        .get_descriptor_set(self.renderpass.get_image_index(&mut self.core)?)
                }
            }
        };
        let render_mesh = RenderMesh {
            push,
            ids: RenderMeshIds {
                index_buffer_id: mesh.indices.buffer_index,
                vertex_buffer_id: mesh.vertices.buffer_index,
                texture_id: match mesh.texture {
                    MeshTexture::RegularTexture(texture) => {
                        renderpass::TextureId::UserTexture(texture.buffer_index)
                    }
                    MeshTexture::Framebuffer(texture) => {
                        renderpass::TextureId::Framebuffer(texture.buffer_index)
                    }
                },
            },
            vertex_buffer: self.vertex_buffers.get(mesh.vertices.buffer_index).unwrap(),
            index_buffer: self.index_buffers.get(mesh.indices.buffer_index).unwrap(),
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
                        .get()
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
                            .get_mut(fb.buffer_index)
                            .unwrap()
                            .get_mut()
                            .framebuffer
                    }
                },
            )
        }
    }
    pub fn finish_render(&mut self) -> Result<()> {
        //the screen frmebuffer must be bound
        if self.bound_framebuffer != BoundFramebuffer::ScreenFramebuffer {
            self.bind_framebuffer(&BoundFramebuffer::ScreenFramebuffer)?;
        }

        let free_data = self.renderpass.submit_draw(&mut self.core)?;
        for id in free_data.iter() {
            match id {
                renderpass::ResourceId::VertexBufferID(id) => {
                    let buffer = self.vertex_buffers.remove(*id).expect("buffer not found");
                    buffer.free(&mut self.core, &mut self.resource_pool)?;
                }

                renderpass::ResourceId::IndexBufferID(id) => {
                    let buffer = self.index_buffers.remove(*id).expect("buffer not found");
                    buffer.free(&mut self.core, &mut self.resource_pool)?;
                }
                ResourceId::UserTexture(_) => (),
                ResourceId::Framebuffer(_) => (),
            }
        }
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
                &self.resource_pool,
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
    pub fn load_shader<P: AsRef<Path>>(&mut self, path: P, shader_name: &str) -> Result<()> {
        self.shaders
            .insert(shader_name.to_string(), load_from_fs(path)?.into());
        Ok(())
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
                tex.drain()
                    .free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free textures");
            }
            for (_idx, mut fb) in self.framebuffer_arena.drain() {
                fb.get_mut()
                    .free(&mut self.core, &mut self.resource_pool)
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
