use anyhow::Result;
use erupt::{vk1_0 as vk, DeviceLoader};
use vk_core::SharedCore;

/// Manages fences and semaphores for every given frame
pub struct FrameSync {
    frames: Vec<Frame>,
    frame_idx: usize,
    prelude: SharedCore,
}

#[derive(Copy, Clone)]
pub struct Frame {
    /// Whether or not rendering has finished
    pub render_finished: vk::Semaphore,
    /// Whether or not this frame is in flight
    pub in_flight_fence: vk::Fence,
}

impl FrameSync {
    pub fn new(prelude: SharedCore, frames_in_flight: usize) -> Result<Self> {
        let frames = (0..frames_in_flight)
            .map(|_| Frame::new(&prelude.device))
            .collect::<Result<_>>()?;

        Ok(Self {
            frames,
            frame_idx: 0,
            prelude,
        })
    }

    pub fn next_frame(&mut self) -> Result<(usize, Frame)> {
        self.frame_idx = (self.frame_idx + 1) % self.frames.len();
        let frame = self.frames[self.frame_idx];
        unsafe {
            self.prelude
                .device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)
                .result()?;
        }
        Ok((self.frame_idx, frame))
    }

    pub fn current_frame(&self) -> usize {
        self.frame_idx
    }
}

impl Drop for FrameSync {
    fn drop(&mut self) {
        for frame in &mut self.frames {
            unsafe {
                self.prelude
                    .device
                    .destroy_semaphore(Some(frame.render_finished), None);
                if !frame.in_flight_fence.is_null() {
                    self.prelude
                        .device
                        .destroy_fence(Some(frame.in_flight_fence), None);
                }
            }
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
}
