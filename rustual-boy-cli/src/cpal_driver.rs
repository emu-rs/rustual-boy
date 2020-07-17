#![allow(dead_code)]

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use rustual_boy_core::sinks::AudioFrame;
use rustual_boy_core::time_source::TimeSource;

use audio_dest::AudioDest;

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::iter::Iterator;

use std::cmp::{self, Ordering};
use cpal::SampleFormat;

pub type CpalDriverError = Cow<'static, str>;

pub struct RingBuffer {
    inner: Box<[i16]>,

    write_pos: usize,
    read_pos: usize,

    samples_read: u64,
}

impl RingBuffer {
    fn push(&mut self, value: i16) {
        self.inner[self.write_pos] = value;

        self.write_pos += 1;
        if self.write_pos >= self.inner.len() {
            self.write_pos = 0;
        }
    }
}

impl Iterator for RingBuffer {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        let ret = self.inner[self.read_pos];

        self.read_pos += 1;
        if self.read_pos >= self.inner.len() {
            self.read_pos = 0;
        }

        self.samples_read += 1;

        Some(ret)
    }
}

struct CpalDriverAudioDest {
    ring_buffer: Arc<Mutex<RingBuffer>>,
}

impl AudioDest for CpalDriverAudioDest {
    fn append(&mut self, buffer: &[AudioFrame]) {
        let mut ring_buffer = self.ring_buffer.lock().unwrap();
        for &(left, right) in buffer {
            ring_buffer.push(left);
            ring_buffer.push(right);
        }
    }
}

struct CpalDriverTimeSource {
    ring_buffer: Arc<Mutex<RingBuffer>>,
    sample_rate: u32,
}

impl TimeSource for CpalDriverTimeSource {
    fn time_ns(&self) -> u64 {
        let ring_buffer = self.ring_buffer.lock().unwrap();
        1_000_000_000 * (ring_buffer.samples_read / 2) / (self.sample_rate as u64)
    }
}

pub struct CpalDriver {
    ring_buffer: Arc<Mutex<RingBuffer>>,
    sample_rate: u32,

    _stream: cpal::Stream,
}

impl CpalDriver {
    pub fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<CpalDriver, CpalDriverError> {
        if desired_latency_ms == 0 {
            return Err(format!("desired_latency_ms must be greater than 0").into());
        }

        let host = cpal::default_host();
        let output_device = host.default_output_device().expect("Failed to get output device");

        let rate_sample_rate_range = |x: &cpal::SupportedStreamConfigRange, y: &cpal::SupportedStreamConfigRange| -> Ordering {
            let sample_rate = cpal::SampleRate(sample_rate);
            if x.max_sample_rate() < sample_rate && y.max_sample_rate() >= sample_rate {
                Ordering::Less
            } else if x.max_sample_rate() >= sample_rate && y.max_sample_rate() < sample_rate {
                Ordering::Greater
            } else if x.max_sample_rate() < sample_rate && y.max_sample_rate() < sample_rate {
                x.max_sample_rate().cmp(&y.max_sample_rate())
            } else {
                // Both max >= sample_rate, rank a lower minimum higher than not
                x.min_sample_rate().cmp(&y.min_sample_rate()).reverse()
            }
        };

        let supported_config = output_device.supported_output_configs()
            .expect("Failed to get supported config list for output")
            .filter(|config| config.channels() == 2)
            .max_by(rate_sample_rate_range)
            .expect("Failed to find format with 2 channels");
        let supported_config = if supported_config.max_sample_rate().0 >= sample_rate {
            let min_sample_rate = supported_config.min_sample_rate().0;
            supported_config.with_sample_rate(cpal::SampleRate(cmp::max(min_sample_rate, sample_rate)))
        } else {
            supported_config.with_max_sample_rate()
        };
        let config = supported_config.config();
        let channel_count = supported_config.channels();


        let buffer_frames = (sample_rate * desired_latency_ms / 1000 * 2) as usize;
        let ring_buffer = Arc::new(Mutex::new(RingBuffer {
            inner: vec![0; buffer_frames].into_boxed_slice(),

            write_pos: 0,
            read_pos: 0,

            samples_read: 0,
        }));

        let mut resampler = LinearResampler::new(sample_rate as _, config.sample_rate.0);
        let read_ring_buffer = ring_buffer.clone();

        let err_handler = |err| eprintln!("Error occurred in output stream: {}", err);
        let out_stream = match supported_config.sample_format() {
            SampleFormat::I16 => output_device.build_output_stream(
                &config,
                move |dst: &mut [i16], _cb_info: &cpal::OutputCallbackInfo| {
                    let mut read_ring_buffer = read_ring_buffer.lock().unwrap();
                    for sample in dst.chunks_mut(channel_count.into()) {
                        for out in sample.iter_mut() {
                            *out = resampler.next(&mut *read_ring_buffer);
                        }
                    }
                },
                err_handler,
            ),
            SampleFormat::U16 => output_device.build_output_stream(
                &config,
                move |dst: &mut [u16], _cb_info: &cpal::OutputCallbackInfo| {
                    let mut read_ring_buffer = read_ring_buffer.lock().unwrap();
                    for sample in dst.chunks_mut(channel_count.into()) {
                        for out in sample.iter_mut() {
                            *out = ((resampler.next(&mut *read_ring_buffer) as i32) + 32768) as u16;
                        }
                    }
                },
                err_handler,
            ),
            SampleFormat::F32 => output_device.build_output_stream(
                &config,
                move |dst: &mut [f32], _cb_info: &cpal::OutputCallbackInfo| {
                    let mut read_ring_buffer = read_ring_buffer.lock().unwrap();
                    for sample in dst.chunks_mut(channel_count.into()) {
                        for out in sample.iter_mut() {
                            *out = (resampler.next(&mut *read_ring_buffer) as f32) / 32768.0;
                        }
                    }
                },
                err_handler,
            ),
        }.expect("Failed to create stream");
        out_stream.play().expect("Failed to play output stream");

        Ok(CpalDriver {
            ring_buffer: ring_buffer,
            sample_rate: sample_rate,

            _stream: out_stream,
        })
    }

    pub fn audio_dest(&self) -> Box<dyn AudioDest> {
        Box::new(CpalDriverAudioDest {
            ring_buffer: self.ring_buffer.clone(),
        })
    }

    pub fn time_source(&self) -> Box<dyn TimeSource> {
        Box::new(CpalDriverTimeSource {
            ring_buffer: self.ring_buffer.clone(),
            sample_rate: self.sample_rate,
        })
    }
}

struct LinearResampler {
    from_sample_rate: u32,
    to_sample_rate: u32,

    current_from_frame: AudioFrame,
    next_from_frame: AudioFrame,
    from_fract_pos: u32,

    current_frame_channel_offset: u32,
}

impl LinearResampler {
    fn new(from_sample_rate: u32, to_sample_rate: u32) -> LinearResampler {
        let sample_rate_gcd = {
            fn gcd(a: u32, b: u32) -> u32 {
                if b == 0 {
                    a
                } else {
                    gcd(b, a % b)
                }
            }

            gcd(from_sample_rate, to_sample_rate)
        };

        LinearResampler {
            from_sample_rate: from_sample_rate / sample_rate_gcd,
            to_sample_rate: to_sample_rate / sample_rate_gcd,

            current_from_frame: (0, 0),
            next_from_frame: (0, 0),
            from_fract_pos: 0,

            current_frame_channel_offset: 0,
        }
    }

    fn next(&mut self, input: &mut dyn Iterator<Item = i16>) -> i16 {
        fn interpolate(a: i16, b: i16, num: u32, denom: u32) -> i16 {
            (((a as i32) * ((denom - num) as i32) + (b as i32) * (num as i32)) / (denom as i32)) as _
        }

        let ret = match self.current_frame_channel_offset {
            0 => interpolate(self.current_from_frame.0, self.next_from_frame.0, self.from_fract_pos, self.to_sample_rate),
            _ => interpolate(self.current_from_frame.1, self.next_from_frame.1, self.from_fract_pos, self.to_sample_rate)
        };

        self.current_frame_channel_offset += 1;
        if self.current_frame_channel_offset >= 2 {
            self.current_frame_channel_offset = 0;

            self.from_fract_pos += self.from_sample_rate;
            while self.from_fract_pos > self.to_sample_rate {
                self.from_fract_pos -= self.to_sample_rate;

                self.current_from_frame = self.next_from_frame;

                let left = input.next().unwrap_or(0);
                let right = input.next().unwrap_or(0);
                self.next_from_frame = (left, right);
            }
        }

        ret
    }
}
