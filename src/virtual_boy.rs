use video_driver::*;
use audio_driver::*;
use rom::*;
use sram::*;
use interconnect::*;
use nvc::*;

pub const CPU_CYCLES_PER_FRAME: usize = 400000;

pub struct VirtualBoy {
    pub interconnect: Interconnect,
    pub cpu: Nvc,
}

impl VirtualBoy {
    pub fn new(rom: Rom, sram: Sram) -> VirtualBoy {
        VirtualBoy {
            interconnect: Interconnect::new(rom, sram),
            cpu: Nvc::new(),
        }
    }

    pub fn step(&mut self, video_driver: &mut VideoDriver, audio_driver: &mut AudioDriver) -> (usize, bool) {
        self.cpu.step(&mut self.interconnect, video_driver, audio_driver)
    }
}