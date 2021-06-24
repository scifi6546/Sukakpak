mod color_buffer;
mod depth_buffer;
mod framebuffer_target;
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use nalgebra::Vector2;

use super::{CommandPool, Core, GraphicsPipeline, ResourcePool, TextureAllocation};
use anyhow::Result;

pub use color_buffer::{AttachmentType, ColorBuffer};
pub use depth_buffer::DepthBuffer;
pub use framebuffer_target::FrameBufferTarget;
pub struct TextureAttachment {
    pub depth_buffer: DepthBuffer,
    pub color_buffer: ColorBuffer,
}
impl TextureAttachment {
    pub fn new(
        core: &mut Core,
        command_pool: &mut CommandPool,
        resource_pool: &mut ResourcePool,
        attachment_type: AttachmentType,
        resolution: Vector2<u32>,
    ) -> Result<Self> {
        let color_buffer = ColorBuffer::new(
            core,
            command_pool,
            resource_pool,
            attachment_type,
            Some(resolution),
        )?;
        let depth_buffer = DepthBuffer::new(core, command_pool, resource_pool, resolution)?;
        Ok(Self {
            color_buffer,
            depth_buffer,
        })
    }
    pub fn free(&mut self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        self.depth_buffer.free(core, resource_pool)?;
        self.color_buffer.free(core);
        Ok(())
    }
}
pub struct Framebuffer {
    pub framebuffer_target: FrameBufferTarget,
    pub resolution: Vector2<u32>,
    pub texture_attachment: TextureAttachment,
}
impl Framebuffer {
    pub fn new(
        core: &mut Core,
        pipeline: &mut GraphicsPipeline,
        texture_attachment: TextureAttachment,
        resolution: Vector2<u32>,
    ) -> Result<Self> {
        let framebuffer_target = FrameBufferTarget::new(
            core,
            pipeline,
            &texture_attachment.color_buffer,
            &texture_attachment.depth_buffer,
            resolution,
        );
        Ok(Self {
            texture_attachment,
            resolution,
            framebuffer_target,
        })
    }
    pub fn free(&mut self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        self.framebuffer_target.free(core);
        self.texture_attachment.free(core, resource_pool)?;
        Ok(())
    }
}
pub struct AttachableFramebuffer {
    pub framebuffer: Framebuffer,
    pub sampler: vk::Sampler,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}
impl AttachableFramebuffer {
    pub fn new(
        core: &mut Core,
        command_pool: &mut CommandPool,
        graphics_pipeline: &mut GraphicsPipeline,
        resource_pool: &mut ResourcePool,
        resolution: Vector2<u32>,
    ) -> Result<Self> {
        let texture_attachment = TextureAttachment::new(
            core,
            command_pool,
            resource_pool,
            AttachmentType::UserFramebuffer,
            resolution,
        )?;
        let framebuffer =
            Framebuffer::new(core, graphics_pipeline, texture_attachment, resolution)?;
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0)
            .max_anisotropy(
                unsafe {
                    core.instance
                        .get_physical_device_properties(core.physical_device)
                }
                .limits
                .max_sampler_anisotropy,
            );
        let sampler = unsafe { core.device.create_sampler(&sampler_info, None) }?;
        let descriptor_sets = framebuffer
            .texture_attachment
            .color_buffer
            .present_image_views
            .iter()
            .map(|view| {
                resource_pool
                    .get_texture_descriptor(core, *view, sampler, vk::ImageLayout::GENERAL)
                    .expect("failed to get descriptor")
            })
            .collect();
        Ok(Self {
            framebuffer,
            sampler,
            descriptor_sets,
        })
    }
    pub fn get_framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }
    pub fn get_descriptor_set(&self, image_index: usize) -> vk::DescriptorSet {
        self.descriptor_sets[image_index]
    }
}
