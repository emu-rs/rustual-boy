extern crate rustual_boy_core;

use rustual_boy_core::{vip, vsu};
use rustual_boy_core::virtual_boy::VirtualBoy;
use rustual_boy_core::rom::Rom;
use rustual_boy_core::sram::Sram;
use rustual_boy_core::time_source::TimeSource;
use rustual_boy_core::sinks::{Sink, VideoFrame};

extern crate rustual_boy_middleware;
use rustual_boy_middleware::{Color, Anaglyphizer, GammaAdjustSink, MostRecentSink};

use input_mapping;
use retro_time_source::RetroTimeSource;
use callbacks::Callbacks;
use callback_sink::CallbackSink;
use system_av_info::{SystemAvInfo, SystemGameGeometry, SystemTiming};

pub struct Context {
    virtual_boy: VirtualBoy,
    emulated_cycles: u64,
    time_source: RetroTimeSource,
}

impl Context {
    pub fn new(rom: Rom, sram: Sram) -> Context {
        Context {
            virtual_boy: VirtualBoy::new(rom, sram),
            emulated_cycles: 0,
            time_source: RetroTimeSource::new(),
        }
    }

    pub fn system_av_info(&self) -> SystemAvInfo {
        SystemAvInfo {
            geometry: SystemGameGeometry {
                base_width: vip::DISPLAY_RESOLUTION_X as u32,
                base_height: vip::DISPLAY_RESOLUTION_Y as u32,
                max_width: vip::DISPLAY_RESOLUTION_X as u32,
                max_height: vip::DISPLAY_RESOLUTION_Y as u32,

                // Optional
                aspect_ratio: 0.0,
            },
            timing: SystemTiming {
                fps: 50.0,
                sample_rate: vsu::SAMPLE_RATE as f64,
            },
        }
    }

    pub fn time_source_mut(&mut self) -> &mut RetroTimeSource {
        &mut self.time_source
    }

    pub fn run_frame(&mut self, callbacks: &'static Callbacks) {
        callbacks.input_poll();
        input_mapping::map_input(callbacks, &mut self.virtual_boy.interconnect.game_pad);

        let mut most_recent_sink: MostRecentSink<VideoFrame> = MostRecentSink::new();
        let mut audio_output_sink = CallbackSink(callbacks);

        // TODO: Record initial time and take difference
        let target_emulated_time_ns = self.time_source.time_ns();
        let target_emulated_cycles = target_emulated_time_ns / 50;

        while self.emulated_cycles < target_emulated_cycles {
            let (num_cycles, _) = self.virtual_boy
                .step(&mut most_recent_sink, &mut audio_output_sink);

            self.emulated_cycles += num_cycles as _;
        }

        if most_recent_sink.has_frame() {
            let video_output_sink = CallbackSink(callbacks);

            let gamma_adjust = GammaAdjustSink::new(video_output_sink, 2.2);

            let mut anaglyphizer = Anaglyphizer::new(gamma_adjust,
                                                     Color::from((255, 0, 0)),
                                                     Color::from((0, 255, 255)));

            let frame = most_recent_sink.into_inner().unwrap();
            anaglyphizer.append(frame);
        }
    }
}
