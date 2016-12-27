use video_driver::*;
use rom::*;
use interconnect::*;
use nvc::*;

pub const CPU_CYCLES_PER_FRAME: usize = 400000;

pub struct VirtualBoy {
    pub interconnect: Interconnect,
    pub cpu: Nvc,
}

impl VirtualBoy {
    pub fn new(rom: Rom) -> VirtualBoy {
        VirtualBoy {
            interconnect: Interconnect::new(rom),
            cpu: Nvc::new(),
        }
    }

    pub fn step(&mut self, video_driver: &mut VideoDriver) -> usize {
        self.cpu.step(&mut self.interconnect, video_driver)
    }
}