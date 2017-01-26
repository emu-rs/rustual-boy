#![allow(dead_code)]

use time_source::*;

use std::time::Instant;

pub struct SystemTimeSource {
    start: Instant,
}

impl SystemTimeSource {
    pub fn new() -> SystemTimeSource {
        SystemTimeSource {
            start: Instant::now(),
        }
    }
}

impl TimeSource for SystemTimeSource {
    fn time_ns(&self) -> u64 {
        let elapsed = self.start.elapsed();
        elapsed.as_secs() * 1000000000 + (elapsed.subsec_nanos() as u64)
    }
}
