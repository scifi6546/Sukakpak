use super::Core;
use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
/// Buffer  contaning semaphores used to start draw calls
pub struct SemaphoreBuffer {
    semaphores: Vec<vk::Semaphore>,
    index: usize,
}
impl SemaphoreBuffer {
    pub fn new(
        starting_semaphore: vk::Semaphore,
        render_finished_semaphore: vk::Semaphore,
    ) -> Self {
        Self {
            semaphores: vec![starting_semaphore],

            index: 0,
        }
    }
    pub fn get_semaphore(&mut self, core: &mut Core) -> Result<SemaphoreGetter> {
        if self.index + 2 <= self.semaphores.len() {
            let old_index = self.index;
            self.index += 1;
            Ok(SemaphoreGetter {
                start_semaphore: self.semaphores[old_index],
                finished_semaphore: self.semaphores[old_index + 1],
            })
        } else {
            let len = (self.index + 2) - self.semaphores.len();
            for _i in 0..len {
                let create_info = vk::SemaphoreCreateInfo::builder().build();
                self.semaphores
                    .push(unsafe { core.device.create_semaphore(&create_info, None) }?);
            }
            let old_index = self.index;
            self.index += 1;
            Ok(SemaphoreGetter {
                start_semaphore: self.semaphores[old_index],
                finished_semaphore: self.semaphores[old_index + 1],
            })
        }
    }
    pub fn last_semaphore(&self) -> vk::Semaphore {
        let len = self.semaphores.len();
        self.semaphores[len - 1]
    }
    pub fn free(&mut self, core: &mut Core) {
        for (i, semaphore) in self.semaphores.iter().enumerate() {
            //skipping first element
            if i != 0 {
                unsafe {
                    core.device.destroy_semaphore(*semaphore, None);
                }
            }
        }
        //unsafe {
        //    device
        //        .device
        //        .destroy_semaphore(self.render_finished_semaphore, None);
        //}
    }
}
pub struct SemaphoreGetter {
    pub start_semaphore: vk::Semaphore,
    pub finished_semaphore: vk::Semaphore,
}
