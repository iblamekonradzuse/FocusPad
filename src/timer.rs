use std::time::{Duration, Instant};

pub struct Timer {
    pub start_time: Option<Instant>,
    pub accumulated_time: Duration,
    pub is_running: bool,
    pub time_offset: Duration, // Debug feature - time offset for time manipulation
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start_time: None,
            accumulated_time: Duration::from_secs(0),
            is_running: false,
            time_offset: Duration::from_secs(0),
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.start_time = Some(Instant::now());
            self.is_running = true;
        }
    }

    pub fn pause(&mut self) {
        if self.is_running {
            if let Some(start) = self.start_time {
                self.accumulated_time += start.elapsed();
                self.start_time = None;
                self.is_running = false;
            }
        }
    }

    pub fn reset(&mut self) {
        self.start_time = None;
        self.accumulated_time = Duration::from_secs(0);
        self.is_running = false;
        // Note: We don't reset the time_offset to allow debug functionality to persist
    }

    // Debug function to add time
    pub fn add_time(&mut self, minutes: f64) {
        let seconds = (minutes * 60.0) as u64;
        self.time_offset += Duration::from_secs(seconds);
    }

    pub fn get_elapsed_time(&self) -> Duration {
        let real_time = if self.is_running {
            if let Some(start) = self.start_time {
                self.accumulated_time + start.elapsed()
            } else {
                self.accumulated_time
            }
        } else {
            self.accumulated_time
        };

        // Add the debug time offset
        real_time + self.time_offset
    }

    pub fn get_elapsed_minutes(&self) -> f64 {
        self.get_elapsed_time().as_secs_f64() / 60.0
    }
}
