extern crate rustual_boy_core;
use rustual_boy_core::time_source::TimeSource;

pub struct RetroTimeSource {
    time_usec: u64,
}

impl RetroTimeSource {
    pub fn new() -> RetroTimeSource {
        RetroTimeSource { time_usec: 0 }
    }

    pub fn append(&mut self, delta_usec: u64) {
        self.time_usec += delta_usec;
    }
}

impl TimeSource for RetroTimeSource {
    fn time_ns(&self) -> u64 {
        self.time_usec * 1000
    }
}
