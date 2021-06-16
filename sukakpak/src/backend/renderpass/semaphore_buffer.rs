use super::Core;
use ash::{version::DeviceV1_0, vk};
/// Buffer  contaning semaphores used to start draw calls
pub struct SemaphoreBuffer {
    semaphores: Vec<vk::Semaphore>,
    render_finished_semaphore: vk::Semaphore,
}
impl SemaphoreBuffer {
    pub fn new(
        starting_semaphore: vk::Semaphore,
        render_finished_semaphore: vk::Semaphore,
    ) -> Self {
        Self {
            semaphores: vec![starting_semaphore],
            render_finished_semaphore,
        }
    }
    // Gets semaphores needed for renderpass. Allocates new ones if they are needed
    pub fn get_semaphores(&mut self, core: &mut Core, size: usize) -> Vec<SemaphoreGetter> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        for _i in self.semaphores.len()..size {
            self.semaphores.push(
                unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
                    .expect("failed to create device"),
            );
        }
        let mut semaphores = (1..self.semaphores.len())
            .map(|i| SemaphoreGetter {
                start_semaphore: self.semaphores[i - 1],
                finished_semaphore: self.semaphores[i],
            })
            .collect::<Vec<_>>();
        let len = self.semaphores.len();
        semaphores.push(SemaphoreGetter {
            start_semaphore: self.semaphores[len - 1],
            finished_semaphore: self.render_finished_semaphore,
        });
        assert_eq!(semaphores.len(), size);
        return semaphores;
    }
    pub fn render_finished_semaphore(&self) -> vk::Semaphore {
        self.render_finished_semaphore
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
