use anyhow::{anyhow, Result};
use ash::vk;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SwapperInfo {
    image_index: u32,
    wait_semaphore: ash::vk::Semaphore,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SwapperRecv {}
pub struct FramebufferSwapper {
    swap_instructions: Sender<SwapperInfo>,
    swap_done: Receiver<SwapperRecv>,
}
impl FramebufferSwapper {
    pub fn swap_framebuffer(
        &mut self,
        image_index: u32,
        wait_semaphore: ash::vk::Semaphore,
    ) -> Result<()> {
        self.swap_instructions.send(SwapperInfo {
            image_index,
            wait_semaphore,
        })?;
        self.swap_done.recv()?;
        Ok(())
    }
}
pub struct FramebufferSwapperMain {
    swap_instructions: Receiver<SwapperInfo>,
    swap_done: Sender<SwapperRecv>,
    swapchain: ash::vk::SwapchainKHR,
    swapchain_loader: ash::extensions::khr::Swapchain,
    surface_loader: AshSurface,
    present_queue: vk::Queue,
}
impl FramebufferSwapperMain {
    fn swap_framebuffer(&mut self, info: SwapperInfo) -> Result<()> {
        let swapchain = [self.swapchain];
        let indices = [info.image_index];
        let wait_semaphore = [info.wait_semaphore];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphore)
            .swapchains(&swapchain)
            .image_indices(&indices);
        unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)?;
        }
        Ok(())
    }
    //runs all swaps
    pub fn run_swapping(&mut self) -> Result<()> {
        loop {
            let recv = self.swap_instructions.try_recv();
            if recv.is_ok() {
                self.swap_framebuffer(recv.ok().unwrap());
                self.swap_done.send(SwapperRecv {})?;
            } else {
                let err = recv.err().unwrap();
                match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => return Err(anyhow!("connection hung up")),
                }
            }
        }
        Ok(())
    }
}
pub fn get_swapper() -> (FramebufferSwapper, FramebufferSwapperMain) {
    todo!()
}
