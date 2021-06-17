use super::Core;
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub enum ShaderStage {
    Fragment,
    Vertex,
    FragmentAndVertex,
}
impl ShaderStage {
    fn to_vk(&self) -> vk::ShaderStageFlags {
        match self {
            Self::Fragment => vk::ShaderStageFlags::FRAGMENT,
            Self::Vertex => vk::ShaderStageFlags::VERTEX,
            Self::FragmentAndVertex => {
                vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX
            }
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorName {
    MeshTexture,
}
#[derive(Clone, Copy)]
pub struct DescriptorDesc {
    pub shader_stage: ShaderStage,
    pub binding: u32,
}
pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    descriptors: HashMap<DescriptorName, vk::DescriptorSetLayout>,
}
impl DescriptorPool {
    pub fn new(core: &Core, descriptors: HashMap<DescriptorName, DescriptorDesc>) -> Result<Self> {
        let pool_sizes = [*vk::DescriptorPoolSize::builder()
            .descriptor_count(descriptors.len() as u32)
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)];
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(descriptors.len() as u32)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
        let descriptor_pool =
            unsafe { core.device.create_descriptor_pool(&pool_create_info, None) }?;
        let descriptors = descriptors
            .iter()
            .map(|(name, descriptor)| {
                (*name, {
                    let layout_binding = [*vk::DescriptorSetLayoutBinding::builder()
                        .binding(descriptor.binding)
                        .descriptor_count(1)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .stage_flags(descriptor.shader_stage.to_vk())];
                    let layout_create_info =
                        vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_binding);
                    unsafe {
                        core.device
                            .create_descriptor_set_layout(&layout_create_info, None)
                    }
                    .expect("failed to create descriptor_set")
                })
            })
            .collect();
        Ok(Self {
            descriptor_pool,
            descriptors,
        })
    }

    pub fn free(&mut self, core: &mut Core) {
        unsafe {
            core.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
