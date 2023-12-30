mod color_buffer;
mod depth_buffer;
mod framebuffer_target;
use ash::{

    Device,Instance,
    vk,
};
use nalgebra::Vector2;

use super::{
    CommandPool, Core, GraphicsPipeline, PipelineType, ResourcePool, ShaderDescription,
    TextureAllocation, TextureDescriptorSets,
};
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
        self.color_buffer.free(core, resource_pool)
    }
}
pub struct Framebuffer {
    pub framebuffer_target: FrameBufferTarget,
    pub resolution: Vector2<u32>,
    pub pipeline: GraphicsPipeline,
    pipeline_type: PipelineType,
    pub texture_attachment: TextureAttachment,
}
impl Framebuffer {
    pub fn new(
        core: &mut Core,
        shader: &ShaderDescription,
        resource_pool: &ResourcePool,
        texture_attachment: TextureAttachment,
        resolution: Vector2<u32>,
        pipeline_type: PipelineType,
    ) -> Result<Self> {
        let mut pipeline = GraphicsPipeline::new(
            core,
            shader,
            &resource_pool.get_descriptor_set_layouts(),
            resolution,
            &texture_attachment.depth_buffer,
            pipeline_type,
        );
        let framebuffer_target =
            FrameBufferTarget::new(core, &mut pipeline, &texture_attachment, resolution);
        Ok(Self {
            texture_attachment,
            resolution,
            pipeline,
            framebuffer_target,
            pipeline_type,
        })
    }
    pub fn rebuild_framebuffer(
        &mut self,
        core: &mut Core,
        resource_pool: &ResourcePool,
        shader: &ShaderDescription,
    ) -> Result<()> {
        self.framebuffer_target.free(core);
        self.pipeline.free(core);
        self.pipeline = GraphicsPipeline::new(
            core,
            shader,
            &resource_pool.get_descriptor_set_layouts(),
            self.resolution,
            &self.texture_attachment.depth_buffer,
            self.pipeline_type,
        );
        self.framebuffer_target = FrameBufferTarget::new(
            core,
            &mut self.pipeline,
            &self.texture_attachment,
            self.resolution,
        );
        Ok(())
    }
    pub fn free(&mut self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        self.framebuffer_target.free(core);
        self.texture_attachment.free(core, resource_pool)?;
        self.pipeline.free(core);
        Ok(())
    }
}
pub struct AttachableFramebuffer {
    pub framebuffer: Framebuffer,
    pub sampler: vk::Sampler,
    pub descriptor_sets: Vec<TextureDescriptorSets>,
}
impl AttachableFramebuffer {
    pub const IMAGE_LAYOUT: vk::ImageLayout = vk::ImageLayout::GENERAL;
    pub fn new(
        core: &mut Core,
        command_pool: &mut CommandPool,
        resource_pool: &mut ResourcePool,
        shader: &ShaderDescription,
        resolution: Vector2<u32>,
    ) -> Result<Self> {
        let texture_attachment = TextureAttachment::new(
            core,
            command_pool,
            resource_pool,
            AttachmentType::UserFramebuffer,
            resolution,
        )?;
        let framebuffer = Framebuffer::new(
            core,
            shader,
            resource_pool,
            texture_attachment,
            resolution,
            PipelineType::OffScreen,
        )?;
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
        let descriptor_sets: Vec<TextureDescriptorSets> = framebuffer
            .texture_attachment
            .color_buffer
            .present_image_views
            .iter()
            .map(|view| {
                resource_pool
                    .get_texture_descriptor(core, *view, sampler, Self::IMAGE_LAYOUT)
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
    pub fn get_descriptor_set(&self, image_index: usize) -> TextureDescriptorSets {
        self.descriptor_sets[image_index].clone()
    }
    pub fn free(&mut self, core: &mut Core, resource_pool: &mut ResourcePool) -> Result<()> {
        unsafe {
            core.device.destroy_sampler(self.sampler, None);
            self.framebuffer.free(core, resource_pool)?;
        }
        Ok(())
    }
}
