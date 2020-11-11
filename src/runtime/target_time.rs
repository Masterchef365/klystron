use std::time::{Duration, Instant};
use std::thread::sleep;

// TODO: This could also be used to measure frame timing...

/// If for some reason vsync doesn't work right (like on my laptop) this can be used to slow down
/// rendering artificially to a slower rate.
pub struct TargetTime {
    target_frame_time: Duration,
    last_frame_start_time: Instant,
}

impl TargetTime {
    /// Create a new target time from frames/sec
    pub fn new(target_fps: u64) -> Self {
        Self {
            target_frame_time: Duration::from_micros(1_000_000 / target_fps),
            last_frame_start_time: Instant::now(),
        }
    }

    /// Start a new frame
    pub fn start_frame(&mut self) {
        self.last_frame_start_time = Instant::now();
    }

    /// End a frame, and wait any slack time we have
    pub fn end_frame(&self) {
        let frame_duration = Instant::now() - self.last_frame_start_time;
        if frame_duration < self.target_frame_time {
            sleep(self.target_frame_time - frame_duration);
        }
    }
}

impl Default for TargetTime {
    fn default() -> Self {
        Self::new(60)
    }
}
