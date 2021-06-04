mod command_pool;
mod depth_buffer;
mod device;
mod framebuffer;
mod index_buffer;
mod mesh;
mod pipeline;
mod present_images;
mod texture;
mod uniform;
mod vertex_buffer;
use ash::{version::DeviceV1_0, vk};
use command_pool::{CommandPool, OffsetData, RenderCollection, RenderMesh, RenderPass};
pub use device::Device;
use framebuffer::Framebuffer;
use generational_arena::{Arena, Index as ArenaIndex};
use std::collections::HashMap;

use depth_buffer::DepthBuffer;
use index_buffer::IndexBuffer;
pub use mesh::{Mesh, MeshID, MeshOffset, MeshOffsetID};
use nalgebra::{Matrix4, Vector2, Vector3};
use pipeline::{
    GraphicsPipeline, ShaderDescription, UniformDescription, VertexBufferDesc, PUSH_SHADER,
};
use present_images::PresentImage;

use texture::{Texture, TextureCreator, TexturePool};
pub use uniform::UniformBuffer;
pub use vertex_buffer::{Vertex, VertexBuffer};

#[derive(Clone, Copy)]
pub struct TextureID {
    index: ArenaIndex,
}
pub struct UniformData {
    pub view_matrix: Matrix4<f32>,
    pub uniforms: HashMap<String, Vec<u8>>,
}
pub struct Context {
    device: Device,
    present_images: PresentImage,
    graphics_pipeline: GraphicsPipeline,
    framebuffer: Framebuffer,
    command_pool: CommandPool,
    render_pass: RenderPass,
    vertex_buffer: VertexBuffer,
    texture_creators: Vec<TextureCreator>,
    texture_pool: TexturePool,
    uniform_buffers: HashMap<String, UniformBuffer>,
    depth_buffer: DepthBuffer,
    textures: Vec<Texture>,
    mesh_arena: Arena<Mesh>,
    mesh_offset_arena: Arena<MeshOffset>,
    texture_arena: Arena<Texture>,
    width: u32,
    height: u32,
    shader_desc: ShaderDescription,
    #[allow(dead_code)]
    window: winit::window::Window,
}
impl Context {
    pub fn new(
        title: &str,
        event_loop: &winit::event_loop::EventLoop<()>,
        width: u32,
        height: u32,
        textures: &[image::RgbaImage],
    ) -> (Self, Vec<TextureID>) {
        let shader_desc = PUSH_SHADER;
        let push_constants = shader_desc
            .push_constants
            .into_iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect();
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(&event_loop)
            .unwrap();
        let mut device = Device::new(&window, width, height);
        let mut present_images = PresentImage::new(&mut device);
        let vertex_buffer = VertexBuffer::new(
            &mut device,
            vec![
                Vertex {
                    position: Vector3::new(-0.5, -0.5, 0.0),
                    uv: Vector2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.5, -0.5, 0.0),
                    uv: Vector2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vector3::new(0.0, 0.5, 0.0),
                    uv: Vector2::new(0.5, 1.0),
                },
            ],
            &shader_desc.vertex_buffer_desc,
        );
        let mat: Matrix4<f32> = Matrix4::identity();
        let uniform_buffers = shader_desc
            .uniforms
            .into_iter()
            .map(|(key, uniform)| {
                (
                    key.to_string(),
                    UniformBuffer::new(
                        &mut device,
                        &present_images,
                        uniform,
                        mat.as_ptr() as *const std::ffi::c_void,
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        let mut layouts = uniform_buffers
            .iter()
            .map(|(_key, buffer)| buffer.get_layout())
            .collect::<Vec<_>>();
        println!("num uniform buffers: {}", layouts.len());
        let mut texture_creators = textures
            .iter()
            .map(|tex| (TextureCreator::new(&mut device), tex))
            .collect::<Vec<_>>();
        println!("texture creators len: {}", texture_creators.len());
        for (creator, _tex) in texture_creators.iter() {
            layouts.push(creator.get_layout());
        }
        println!("total descriptor set layout len: {}", layouts.len());
        let mut command_pool = CommandPool::new(&mut device);
        let depth_buffer = DepthBuffer::new(&mut device, &mut command_pool, width, height);
        let mut graphics_pipeline = GraphicsPipeline::new(
            &mut device,
            &shader_desc,
            &vertex_buffer,
            layouts,
            &push_constants,
            width,
            height,
            &depth_buffer,
        );

        let framebuffer = Framebuffer::new(
            &mut device,
            &mut present_images,
            &mut graphics_pipeline,
            &depth_buffer,
            width,
            height,
        );
        let (texture_pool, mut textures) = TexturePool::new(
            &mut device,
            &mut command_pool,
            &texture_creators,
            &present_images,
        );
        println!("textures len: {}", textures.len());
        let render_pass = RenderPass::new(&mut device, &command_pool, &framebuffer);
        let mut texture_arena = Arena::new();
        let texture_ids = textures
            .drain(..)
            .map(|tex| TextureID {
                index: texture_arena.insert(tex),
            })
            .collect();

        (
            Self {
                device,
                present_images,
                graphics_pipeline,
                framebuffer,
                command_pool,
                vertex_buffer,
                uniform_buffers,
                textures,
                render_pass,
                texture_pool,
                texture_creators: texture_creators
                    .drain(..)
                    .map(|(creator, _tex)| creator)
                    .collect(),
                window,
                width,
                height,
                mesh_arena: Arena::new(),
                mesh_offset_arena: Arena::new(),
                texture_arena,
                depth_buffer,
                shader_desc,
            },
            texture_ids,
        )
    }
    pub fn render_frame(&mut self, mesh: &[(MeshOffsetID, UniformData)]) {
        let mut render_collection = RenderCollection::default();
        for (id, uniform_data) in mesh.iter() {
            let offset = self
                .mesh_offset_arena
                .get(id.index)
                .expect("invalid mesh id");
            let mesh = self
                .mesh_arena
                .get(offset.mesh.index)
                .expect("invalid mesh id");
            render_collection.push(mesh.to_render_mesh(
                uniform_data.view_matrix,
                HashMap::new(),
                &self.texture_arena,
                &offset,
            ));
        }
        unsafe {
            self.render_pass.render_frame(
                &mut self.device,
                &self.framebuffer,
                &self.graphics_pipeline,
                self.width,
                self.height,
                &mut self.uniform_buffers,
                &render_collection,
            );
        }
    }
    pub fn new_mesh(
        &mut self,
        texture: TextureID,
        verticies: Vec<Vertex>,
        indicies: Vec<u32>,
    ) -> MeshOffsetID {
        MeshOffsetID {
            index: self.mesh_offset_arena.insert(MeshOffset {
                mesh: MeshID {
                    index: self.mesh_arena.insert(Mesh::new(
                        &mut self.device,
                        &mut self.command_pool,
                        &self.shader_desc.vertex_buffer_desc,
                        texture,
                        verticies,
                        indicies,
                    )),
                },
                vertex_buffer_offset: 0,
                index_buffer_offset: 0,
                texture,
            }),
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        self.render_pass.wait_idle(&mut self.device);
        self.depth_buffer.free(&mut self.device);
        for (_idx, texture) in self.texture_arena.iter_mut() {
            texture.free(&mut self.device, &self.texture_pool);
        }
        for texture in self.textures.iter_mut() {
            texture.free(&mut self.device, &self.texture_pool);
        }

        self.texture_pool.free(&mut self.device);
        for creator in self.texture_creators.iter() {
            creator.free(&mut self.device);
        }
        self.render_pass.free(&mut self.device);
        self.command_pool.free(&mut self.device);
        self.framebuffer.free(&mut self.device);
        self.graphics_pipeline.free(&mut self.device);
        for (_k, buffer) in self.uniform_buffers.iter_mut() {
            buffer.free(&mut self.device);
        }
        self.vertex_buffer.free(&mut self.device);
        for (_idx, mesh) in self.mesh_arena.iter_mut() {
            mesh.free(&mut self.device);
        }
        self.present_images.free(&mut self.device);
        self.device.free();
    }
}
pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
pub fn copy_buffer(
    device: &mut Device,
    command_pool: &mut CommandPool,
    src_buffer: &vk::Buffer,
    dst_buffer: &vk::Buffer,
    buffer_size: u64,
) {
    unsafe {
        let copy_command = command_pool.create_onetime_buffer(device);
        let copy_region = [*vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(buffer_size)];
        copy_command.device.device.cmd_copy_buffer(
            copy_command.command_buffer[0],
            *src_buffer,
            *dst_buffer,
            &copy_region,
        );
    }
}
pub struct FreeChecker {
    freed: bool,
}
impl FreeChecker {
    pub fn free(&mut self) {
        if self.freed == true {
            panic!("already freed")
        } else {
            self.freed = true;
        }
    }
}
impl Default for FreeChecker {
    fn default() -> Self {
        Self { freed: false }
    }
}
trait DescriptorSets {
    fn get_layout(&self) -> vk::DescriptorSetLayout;
}
