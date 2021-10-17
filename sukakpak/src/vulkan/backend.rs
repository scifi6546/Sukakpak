use anyhow::{anyhow, Context as AContext, Result};
use ash::vk;

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
use super::CreateInfo;
use super::{VertexComponent, VertexLayout};
use command_pool::CommandPool;
use framebuffer::{
    AttachableFramebuffer, AttachmentType, DepthBuffer, FrameBufferTarget, Framebuffer,
    TextureAttachment,
};
use generational_arena::{Arena, Index as ArenaIndex};
use pipeline::{basic_shader, GraphicsPipeline, PipelineType, ShaderDescription};
use ref_counter::RefCounter;
use render_core::Core;
mod pipeline;
use renderpass::{ClearOp, RenderMesh, RenderMeshIds, RenderPass, ResourceId};
use resource_pool::{
    DescriptorDesc, IndexBufferAllocation, ResourcePool, TextureAllocation, TextureDescriptorSets,
    VertexBufferAllocation,
};
use std::collections::HashSet;
use std::{collections::HashMap, path::Path};

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Rendering to framebuffer {fb:?}")]
    RenderingBoundFramebuffer { fb: BoundFramebuffer },
    #[error("Shader: {shader:} not found")]
    ShaderNotFound { shader: String },
}
unsafe impl Send for Backend {}
pub struct Backend {
    #[allow(dead_code)]
    shaders: HashMap<String, ShaderDescription>,
    window: winit::window::Window,
    models: Arena<Model>,
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
/// Complete Mesh
pub struct Model {
    vertices: VertexBufferAllocation,
    indices: IndexBufferAllocation,
    texture: MeshTexture,
}
#[derive(Clone, Copy, Debug)]
pub struct MeshID {
    buffer_index: ArenaIndex,
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
        create_info: CreateInfo,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Result<Self> {
        let shaders = [("basic".to_string(), basic_shader())]
            .iter()
            .cloned()
            .collect();
        let main_shader = basic_shader();
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
            models: Arena::new(),
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
        self.incr_texture_refrences(&texture);

        let vertices =
            self.resource_pool
                .allocate_vertex_buffer(&mut self.core, verticies, vertex_layout)?;
        let indices = self.resource_pool.allocate_index_buffer(
            &mut self.core,
            &mut self.command_pool,
            indicies,
        )?;

        Ok(MeshID {
            buffer_index: self.models.insert(Model {
                vertices,
                indices,
                texture,
            }),
        })
    }
    /// Decrements refrences on mesh texture, and marks for freeing if refrences is zero
    /// Preconditions:
    /// Mesh texture is valid and has more then 0 refrences
    fn decr_texture_refrences(&mut self, texture: &MeshTexture) {
        match texture {
            MeshTexture::RegularTexture(id) => {
                let texture_ref = self.textures.get_mut(id.buffer_index).unwrap();
                texture_ref.decr_refrence();
            }
            MeshTexture::Framebuffer(id) => {
                let texture_ref = self.framebuffer_arena.get_mut(id.buffer_index).unwrap();
                texture_ref.decr_refrence();
            }
        };
    }
    /// Increments refrences on mesh texture
    /// Preconditions:
    /// None
    pub fn incr_texture_refrences(&mut self, texture: &MeshTexture) {
        match texture {
            MeshTexture::RegularTexture(id) => {
                let texture_option = self.textures.get_mut(id.buffer_index);
                if texture_option.is_none() {
                    panic!("texture : {:?} does not exist", id);
                }
                let texture_ref = texture_option.unwrap();
                texture_ref.incr_refrence();
            }
            MeshTexture::Framebuffer(id) => {
                let framebuffer_ref = self.framebuffer_arena.get_mut(id.buffer_index).unwrap();
                framebuffer_ref.incr_refrence();
            }
        };
    }
    pub fn bind_texture(&mut self, mesh_id: &mut MeshID, texture: MeshTexture) -> Result<()> {
        let old_texture = self.models.get(mesh_id.buffer_index).unwrap().texture;
        self.decr_texture_refrences(&old_texture);
        self.models.get_mut(mesh_id.buffer_index).unwrap().texture = texture;
        self.incr_texture_refrences(&texture);
        Ok(())
    }
    pub fn allocate_texture(&mut self, texture: &RgbaImage) -> Result<TextureID> {
        let texture = TextureID {
            buffer_index: self.textures.insert(RefCounter::new(
                self.resource_pool.allocate_texture(
                    &mut self.core,
                    &mut self.command_pool,
                    texture,
                )?,
                0,
            )),
        };
        Ok(texture)
    }
    /// Lazily frees textures once the texture is no longer in use
    pub fn free_texture(&mut self, tex: MeshTexture) -> Result<()> {
        self.to_free_textures.insert(tex);
        Ok(())
    }
    /// Scans resources and frees all resources that need to be freed
    pub fn collect_garbage(&mut self) -> Result<()> {
        let mut freed_textures: Vec<MeshTexture> = vec![];
        for tex in self.to_free_textures.iter() {
            match tex {
                MeshTexture::RegularTexture(id) => {
                    let num_refrences = self.textures.get(id.buffer_index).unwrap().refrences();
                    if num_refrences == 0
                        && !self
                            .renderpass
                            .is_resource_used(&ResourceId::UserTexture(id.buffer_index))
                    {
                        freed_textures.push(*tex);
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
                        freed_textures.push(*tex);
                        self.framebuffer_arena
                            .remove(id.buffer_index)
                            .unwrap()
                            .drain()
                            .free(&mut self.core, &mut self.resource_pool)?;
                    }
                }
            }
        }
        for tex in freed_textures.iter() {
            if self.to_free_textures.remove(tex) == false {
                panic!("invalid state texture not in free list")
            }
        }
        Ok(())
    }
    pub fn build_framebuffer(&mut self, resolution: Vector2<u32>) -> Result<FramebufferID> {
        let framebuffer = FramebufferID {
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
        };
        self.to_free_textures
            .insert(MeshTexture::Framebuffer(framebuffer));

        Ok(framebuffer)
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
    pub fn free_mesh(&mut self, mesh_id: &MeshID) -> Result<()> {
        let texture = self.models.get(mesh_id.buffer_index).unwrap().texture;
        let ids = RenderMeshIds {
            mesh_id: mesh_id.buffer_index,
            texture_id: match texture {
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

    pub fn draw_mesh(&mut self, push: Vec<u8>, mesh_id: &MeshID) -> Result<()> {
        let mesh = self.models.get(mesh_id.buffer_index).unwrap();
        let descriptor_set = match mesh.texture {
            MeshTexture::RegularTexture(texture) => self
                .textures
                .get(texture.buffer_index)
                .unwrap()
                .get()
                .descriptor_sets
                .clone(),
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
                mesh_id: mesh_id.buffer_index,
                texture_id: match mesh.texture {
                    MeshTexture::RegularTexture(texture) => {
                        renderpass::TextureId::UserTexture(texture.buffer_index)
                    }
                    MeshTexture::Framebuffer(texture) => {
                        renderpass::TextureId::Framebuffer(texture.buffer_index)
                    }
                },
            },
            vertex_buffer: &mesh.vertices,
            index_buffer: &mesh.indices,
        };
        let descriptor_set_arr = [
            descriptor_set.texture_descriptor_set,
            descriptor_set.sampler_descriptor_set,
        ];
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
            &descriptor_set_arr,
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
                ResourceId::Mesh(id) => {
                    {
                        let tex = self.models.get(*id).unwrap().texture.clone();
                        self.decr_texture_refrences(&tex);
                    }
                    let model = self.models.remove(*id).unwrap();

                    model
                        .indices
                        .free(&mut self.core, &mut self.resource_pool)?;
                    model
                        .vertices
                        .free(&mut self.core, &mut self.resource_pool)?;
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
                self.resize_renderer(new_size)?;
                Ok(())
            } else if r == vk::Result::SUBOPTIMAL_KHR {
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
    pub fn load_shader(&mut self, shader_data: &str, shader_name: &str) -> Result<()> {
        let shader = ass_lib::vk::Shader::from_json_str(shader_data)
            .with_context(|| format!("failed to load shader {}", shader_name))?;
        self.shaders.insert(shader_name.to_string(), shader.into());
        Ok(())
    }
    /// Validates state, panics if state is invalid
    /// Warning: may be slow
    pub fn check_state(&mut self) {
        let mut num_correct_refrences: HashMap<MeshTexture, usize> = HashMap::new();
        for (_id, mesh) in self.models.iter() {
            if num_correct_refrences.contains_key(&mesh.texture) {
                let num_ref = num_correct_refrences.get_mut(&mesh.texture).unwrap();
                *num_ref += 1;
            } else {
                num_correct_refrences.insert(mesh.texture, 1);
            }
            match mesh.texture {
                MeshTexture::RegularTexture(id) => {
                    if self.textures.get(id.buffer_index).is_none() {
                        panic!("texture: {:?} does not exist", id)
                    }
                }
                MeshTexture::Framebuffer(id) => {
                    if self.framebuffer_arena.get(id.buffer_index).is_none() {
                        panic!("framebuffer: {:?} does not exist", id)
                    }
                }
            }
        }
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.renderpass.wait_idle(&mut self.core);
        unsafe {
            for (_idx, model) in self.models.drain() {
                model
                    .vertices
                    .free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free vertex buffer");
                model
                    .indices
                    .free(&mut self.core, &mut self.resource_pool)
                    .expect("failed to free index buffer");
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
