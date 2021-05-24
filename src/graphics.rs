mod command_pool;
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
use command_pool::{CommandPool, RenderMesh, RenderPass};
pub use device::Device;
use framebuffer::Framebuffer;
use generational_arena::{Arena, Index as ArenaIndex};

use index_buffer::IndexBuffer;
pub use mesh::Mesh;
use nalgebra::{Matrix4, Vector2, Vector3};
use pipeline::GraphicsPipeline;
use present_images::PresentImage;
use texture::{Texture, TextureCreator, TexturePool};
pub use uniform::UniformBuffer;
pub use vertex_buffer::{Vertex, VertexBuffer};
#[derive(Clone, Copy)]
pub struct MeshID {
    index: ArenaIndex,
}
#[derive(Clone, Copy)]
pub struct TextureID {
    index: ArenaIndex,
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
    uniform_buffer: UniformBuffer<{ std::mem::size_of::<Matrix4<f32>>() }>,
    textures: Vec<Texture>,
    mesh_arena: Arena<Mesh>,
    texture_arena: Arena<Texture>,
    width: u32,
    height: u32,
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
        );
        let mat: Matrix4<f32> = Matrix4::identity();
        let uniform_buffer = UniformBuffer::new(
            &mut device,
            &present_images,
            mat.as_ptr() as *const std::ffi::c_void,
        );
        let mut layouts = vec![uniform_buffer.get_layout()];
        let mut texture_creators = textures
            .iter()
            .map(|tex| (TextureCreator::new(&mut device), tex))
            .collect::<Vec<_>>();
        for (creator, _tex) in texture_creators.iter() {
            layouts.push(creator.get_layout());
        }

        let mut graphics_pipeline =
            GraphicsPipeline::new(&mut device, &vertex_buffer, layouts, width, height);

        let framebuffer = Framebuffer::new(
            &mut device,
            &mut present_images,
            &mut graphics_pipeline,
            width,
            height,
        );
        let mut command_pool = CommandPool::new(&mut device);
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
                uniform_buffer,
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
                texture_arena,
            },
            texture_ids,
        )
    }
    pub fn render_frame(&mut self, mesh: &[(MeshID, *const std::ffi::c_void)]) {
        let mut render_meshes = vec![];
        render_meshes.reserve(mesh.len());
        for (id, uniform_data) in mesh.iter() {
            let mesh = self.mesh_arena.get(id.index).expect("invalid mesh id");
            render_meshes.push(mesh.to_render_mesh(*uniform_data, &self.texture_arena));
        }
        unsafe {
            self.render_pass.render_frame(
                &mut self.device,
                &self.framebuffer,
                &self.graphics_pipeline,
                self.width,
                self.height,
                &mut self.uniform_buffer,
                &mut render_meshes,
            );
        }
    }
    pub fn new_mesh(
        &mut self,
        texture: TextureID,
        verticies: Vec<Vertex>,
        indicies: Vec<u32>,
    ) -> MeshID {
        MeshID {
            index: self.mesh_arena.insert(Mesh::new(
                &mut self.device,
                &mut self.command_pool,
                texture,
                verticies,
                indicies,
            )),
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        self.render_pass.wait_idle(&mut self.device);
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
        self.uniform_buffer.free(&mut self.device);
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
