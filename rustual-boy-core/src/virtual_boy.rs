use sinks::*;
use rom::*;
use sram::*;
use interconnect::*;
use v810::*;

pub struct VirtualBoy {
    pub interconnect: Interconnect,
    pub cpu: V810,
}

impl VirtualBoy {
    pub fn new(rom: Rom, sram: Sram) -> VirtualBoy {
        VirtualBoy {
            interconnect: Interconnect::new(rom, sram),
            cpu: V810::new(),
        }
    }

    pub fn step(&mut self, video_frame_sink: &mut dyn Sink<VideoFrame>, audio_frame_sink: &mut dyn Sink<AudioFrame>) -> (u32, bool) {
        let ret = self.cpu.step(&mut self.interconnect);

        if let Some(exception_code) = self.interconnect.cycles(ret.0, video_frame_sink, audio_frame_sink) {
            self.cpu.request_interrupt(exception_code);
        }

        ret
    }
}
