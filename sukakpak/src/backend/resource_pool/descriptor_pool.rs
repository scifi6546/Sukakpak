use super::Core;
use anyhow::{anyhow, Result};
use ash::{version::DeviceV1_0, vk};
use std::collections::HashMap;
use thiserror::Error;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DescriptorName {
    MeshTexture,
}
#[derive(Error, Debug)]
pub enum DescriptorError {
    #[error("Descriptor {0:?} not found")]
    DescriptorSetLayoutNotFound(DescriptorName),
    #[error("Descriptor Pool Full")]
    DescriptorPoolFull,
}
#[derive(Clone, Copy)]
pub struct DescriptorDesc {
    pub layout_binding: vk::DescriptorSetLayoutBinding,
}

/// TODO: HANDLE REMAPPING
pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    pub descriptors: HashMap<DescriptorName, (vk::DescriptorSetLayout, DescriptorDesc)>,
    num_descriptors_allocated: usize,
}
impl DescriptorPool {
    const MAX_SETS: u32 = 100;
    pub fn new(
        core: &Core,
        pool_type: vk::DescriptorType,
        descriptors: HashMap<DescriptorName, DescriptorDesc>,
    ) -> Result<Self> {
        let pool_sizes = [*vk::DescriptorPoolSize::builder()
            .descriptor_count(Self::MAX_SETS)
            .ty(pool_type)];
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(Self::MAX_SETS)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
        let descriptor_pool =
            unsafe { core.device.create_descriptor_pool(&pool_create_info, None) }?;
        let descriptors = descriptors
            .iter()
            .map(|(name, descriptor)| {
                (name.clone(), {
                    let layout_binding = [descriptor.layout_binding];
                    println!("making layout: {:?}", layout_binding);
                    let layout_create_info =
                        vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_binding);
                    let layouts = [unsafe {
                        core.device
                            .create_descriptor_set_layout(&layout_create_info, None)
                    }
                    .expect("failed to create descriptor_set")];
                    (layouts[0], descriptor.clone())
                })
            })
            .collect();
        Ok(Self {
            descriptor_pool,
            descriptors,
            num_descriptors_allocated: 0,
        })
    }
    pub fn get_descriptor_layouts(&self) -> Vec<vk::DescriptorSetLayout> {
        self.descriptors
            .iter()
            .map(|(_key, (layout, _desc))| *layout)
            .collect()
    }
    pub unsafe fn allocate_descriptor_set(
        &mut self,
        core: &mut Core,
        descriptor_name: &DescriptorName,
    ) -> Result<Vec<vk::DescriptorSet>> {
        if self.num_descriptors_allocated + 1 >= Self::MAX_SETS as usize {
            return Err(anyhow!("{}", DescriptorError::DescriptorPoolFull));
        }
        if let Some((layout, _desc)) = self.descriptors.get(descriptor_name) {
            let layouts = [*layout];
            let alloc_info = vk::DescriptorSetAllocateInfo::builder()
                .set_layouts(&layouts)
                .descriptor_pool(self.descriptor_pool);
            let sets = core.device.allocate_descriptor_sets(&alloc_info)?;
            Ok(sets)
        } else {
            Err(anyhow!(
                "{}",
                DescriptorError::DescriptorSetLayoutNotFound(descriptor_name.clone())
            ))
        }
    }
    pub fn get_descriptor_desc(&self, name: &DescriptorName) -> Option<DescriptorDesc> {
        if let Some((_layout, desc)) = self.descriptors.get(name) {
            Some(desc.clone())
        } else {
            None
        }
    }
    pub fn free(&mut self, core: &mut Core) -> Result<()> {
        unsafe {
            for (_name, (layout, _desc)) in self.descriptors.iter() {
                core.device.destroy_descriptor_set_layout(*layout, None);
            }
            core.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
        Ok(())
    }
}
