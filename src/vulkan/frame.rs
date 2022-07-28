use super::*;

pub struct Frames<const N: usize> {
    current_frame: usize,
    pub cmd_bufs: [vk::CommandBuffer; N],
    pub render_semaphores: [vk::Semaphore; N],
    pub present_semaphores: [vk::Semaphore; N],
    pub frame_fences: [vk::Fence; N],
}

impl<const N: usize> Frames<N> {
    pub fn create(bvk: &BabyVulkan, cmd_pool: vk::CommandPool) -> Option<Self> {
        let mut frames = Frames {
            current_frame: 0,
            cmd_bufs: [vk::CommandBuffer::null(); N],
            render_semaphores: [vk::Semaphore::null(); N],
            present_semaphores: [vk::Semaphore::null(); N],
            frame_fences: [vk::Fence::null(); N],
        };

        for i in 0..N {
            frames.cmd_bufs[i] = bvk.create_primary_command_buffer(cmd_pool)?;
            frames.render_semaphores[i] = bvk.create_semaphore()?;
            frames.present_semaphores[i] = bvk.create_semaphore()?;
            frames.frame_fences[i] = bvk.create_fence()?;
        }

        Some(frames)
    }

    pub fn destroy(&self, bvk: &BabyVulkan) {
        for i in 0..N {
            unsafe {
                bvk.dev.destroy_semaphore(self.render_semaphores[i], None);
                bvk.dev.destroy_semaphore(self.present_semaphores[i], None);
                bvk.dev.destroy_fence(self.frame_fences[i], None);
            }
        }
    }

    pub fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn advance(&mut self) {
        self.current_frame = (self.current_frame + 1) % N
    }
}
