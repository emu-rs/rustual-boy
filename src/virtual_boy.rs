use video_driver::*;
use audio_driver::*;
use rom::*;
use sram::*;
use interconnect::*;
use nvc::*;

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

    pub fn step(&mut self, video_driver: &mut VideoDriver, audio_driver: &mut AudioDriver) -> bool {
        let (num_cycles, trigger_watchpoint) = self.cpu.step(&mut self.interconnect);

        if let Some(exception_code) = self.interconnect.cycles(num_cycles, video_driver, audio_driver) {
            self.cpu.request_interrupt(exception_code);
        }

        trigger_watchpoint
    }
}