use super::Device;
use ash::version::DeviceV1_0;
use ash::{util::*, vk};
use std::io::Cursor;
pub struct GraphicsPipeline {
    fragment_shader: vk::ShaderModule,
    vertex_shader: vk::ShaderModule,
}
impl GraphicsPipeline {
    pub fn new(device: &mut Device, screen_width: u32, screen_height: u32) -> Self {
        let frag_shader_data =
            read_spv(&mut Cursor::new(include_bytes!("../shaders/main.frag.spv")))
                .expect("failed to create shader");
        let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_shader_data);
        let fragment_shader = unsafe {
            device
                .device
                .create_shader_module(&frag_shader_info, None)
                .expect("failed to create shader")
        };

        let vert_shader_data =
            read_spv(&mut Cursor::new(include_bytes!("../shaders/main.vert.spv")))
                .expect("failed to create shader");
        let vert_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vert_shader_data);
        let vertex_shader = unsafe {
            device
                .device
                .create_shader_module(&frag_shader_info, None)
                .expect("failed to create shader")
        };
        GraphicsPipeline {
            fragment_shader,
            vertex_shader,
        }
    }
    pub fn free(&mut self, device: &mut Device) {
        unsafe {
            device
                .device
                .destroy_shader_module(self.fragment_shader, None);
            device
                .device
                .destroy_shader_module(self.vertex_shader, None);
        }
    }
}
