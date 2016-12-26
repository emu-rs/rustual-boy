use video_driver::*;
use rom::*;
use interconnect::*;
use nvc::*;

const CPU_CYCLES_PER_FRAME: usize = 400000;

pub struct VirtualBoy {
    pub interconnect: Interconnect,
    pub cpu: Nvc,

    frame_cycles: usize,
}

impl VirtualBoy {
    pub fn new(rom: Rom) -> VirtualBoy {
        VirtualBoy {
            interconnect: Interconnect::new(rom),
            cpu: Nvc::new(),

            frame_cycles: 0,
        }
    }

    pub fn step(&mut self, video_driver: &mut VideoDriver) {
        self.frame_cycles += self.cpu.step(&mut self.interconnect, video_driver);
        if self.frame_cycles >= CPU_CYCLES_PER_FRAME {
            self.frame_cycles -= CPU_CYCLES_PER_FRAME;
        }
    }

    pub fn step_frame(&mut self, video_driver: &mut VideoDriver) {
        while self.frame_cycles < CPU_CYCLES_PER_FRAME {
            self.frame_cycles += self.cpu.step(&mut self.interconnect, video_driver);
        }
        self.frame_cycles -= CPU_CYCLES_PER_FRAME;
    }
}