use super::Core;
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::{cmp::max, collections::HashMap};
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
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum DescriptorName {
    MeshTexture,
    Uniform(String),
}
#[derive(Clone, Copy)]
pub struct DescriptorDesc {
    pub layout_binding: vk::DescriptorSetLayoutBinding,
}
pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    pub descriptors: HashMap<DescriptorName, (vk::DescriptorSetLayout, vk::DescriptorSet)>,
}
impl DescriptorPool {
    pub fn new(
        core: &Core,
        pool_type: vk::DescriptorType,
        descriptors: HashMap<DescriptorName, DescriptorDesc>,
    ) -> Result<Self> {
        let pool_size = max(descriptors.len(), 1) as u32;
        let pool_sizes = [*vk::DescriptorPoolSize::builder()
            .descriptor_count(pool_size)
            .ty(pool_type)];
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(pool_size)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
        let descriptor_pool =
            unsafe { core.device.create_descriptor_pool(&pool_create_info, None) }?;
        let descriptors = descriptors
            .iter()
            .map(|(name, descriptor)| {
                (name.clone(), {
                    let layout_binding = [descriptor.layout_binding];
                    let layout_create_info =
                        vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_binding);
                    let layouts = [unsafe {
                        core.device
                            .create_descriptor_set_layout(&layout_create_info, None)
                    }
                    .expect("failed to create descriptor_set")];
                    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
                        .descriptor_pool(descriptor_pool)
                        .set_layouts(&layouts);
                    let descriptor_set = unsafe {
                        core.device
                            .allocate_descriptor_sets(&descriptor_set_alloc_info)
                    }
                    .expect("failed to alloc descriptor set");
                    (layouts[0], descriptor_set[0])
                })
            })
            .collect();
        Ok(Self {
            descriptor_pool,
            descriptors,
        })
    }
    pub fn get_descriptor_pools(&self) -> Vec<vk::DescriptorSetLayout> {
        self.descriptors
            .iter()
            .map(|(_key, (layout, _set))| *layout)
            .collect()
    }
    pub fn free(&mut self, core: &mut Core) -> Result<()> {
        unsafe {
            for (_name, (layout, set)) in self.descriptors.iter() {
                core.device
                    .free_descriptor_sets(self.descriptor_pool, &[*set])?;
                core.device.destroy_descriptor_set_layout(*layout, None);
            }
            core.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
        Ok(())
    }
}
