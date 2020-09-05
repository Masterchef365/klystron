use anyhow::Result;
use erupt::{vk1_0 as vk, DeviceLoader};

/// Manages fences and semaphores for every given frame
pub struct FrameSync {
    frames: Vec<Frame>,
    frame_idx: usize,
    freed: bool,
}

pub struct Frame {
    /// Whether or not rendering has finished
    pub render_finished: vk::Semaphore,
    /// Whether or not this frame is in flight
    pub in_flight_fence: vk::Fence,
}

impl FrameSync {
    pub fn new(device: &DeviceLoader, frames_in_flight: usize) -> Result<Self> {
        let frames = (0..frames_in_flight)
            .map(|_| Frame::new(device))
            .collect::<Result<_>>()?;

        Ok(Self {
            frames,
            freed: false,
            frame_idx: 0,
        })
    }

    pub fn next_frame(&mut self, device: &DeviceLoader) -> Result<(usize, &mut Frame)> {
        self.frame_idx = (self.frame_idx + 1) % self.frames.len();
        let frame = &mut self.frames[self.frame_idx];
        unsafe {
            device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX).result()?;
        }
        Ok((self.frame_idx, frame))
    }

    pub fn free(&mut self, device: &DeviceLoader) {
        for frame in &mut self.frames {
            frame.free(device);
        }
        self.freed = true;
    }
}

impl Drop for FrameSync {
    fn drop(&mut self) {
        if !self.freed {
            panic!("FrameSync dropped before its free() method was called!");
        }
    }
}

impl Frame {
    pub fn new(device: &DeviceLoader) -> Result<Self> {
        unsafe {
            let create_info = vk::SemaphoreCreateInfoBuilder::new();
            let render_finished = device.create_semaphore(&create_info, None, None).result()?;

            let create_info =
                vk::FenceCreateInfoBuilder::new().flags(vk::FenceCreateFlags::SIGNALED);
            let in_flight_fence = device.create_fence(&create_info, None, None).result()?;
            Ok(Self {
                in_flight_fence,
                render_finished,
            })
        }
    }

    pub fn free(&mut self, device: &DeviceLoader) {
        unsafe {
            device.destroy_semaphore(Some(self.render_finished), None);
            if !self.in_flight_fence.is_null() {
                device.destroy_fence(Some(self.in_flight_fence), None);
            }
        }
    }
}
