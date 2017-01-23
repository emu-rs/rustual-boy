use cpal::{EventLoop, Voice, UnknownTypeBuffer, get_default_endpoint};

use futures::stream::Stream;
use futures::task::{self, Executor, Run};

use audio_buffer_sink::*;
use time_source::*;

use std::borrow::Cow;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::iter::Iterator;
use std::thread::{self, JoinHandle};

pub type CpalDriverError = Cow<'static, str>;

pub struct RingBuffer {
    inner: VecDeque<i16>,
    samples_read: u64,
}

impl Iterator for RingBuffer {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        match self.inner.pop_front() {
            Some(x) => {
                self.samples_read += 1;
                Some(x)
            }
            _ => None
        }
    }
}

struct CpalDriverBufferSink {
    ring_buffer: Arc<Mutex<RingBuffer>>,
}

impl AudioBufferSink for CpalDriverBufferSink {
    fn append(&mut self, buffer: &[(i16, i16)]) {
        let mut ring_buffer = self.ring_buffer.lock().unwrap();
        ring_buffer.inner.extend(buffer.into_iter().flat_map(|&(left, right)| vec![left, right].into_iter()));
    }
}

struct CpalDriverTimeSource {
    ring_buffer: Arc<Mutex<RingBuffer>>,
    sample_rate: usize,
}

impl TimeSource for CpalDriverTimeSource {
    fn time_ns(&self) -> u64 {
        let ring_buffer = self.ring_buffer.lock().unwrap();
        1_000_000_000 * (ring_buffer.samples_read / 2) / (self.sample_rate as u64)
    }
}

struct CpalDriverExecutor;

impl Executor for CpalDriverExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

pub struct CpalDriver {
    ring_buffer: Arc<Mutex<RingBuffer>>,
    sample_rate: usize,

    _voice: Voice,
    _join_handle: JoinHandle<()>,
}

impl CpalDriver {
    pub fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<CpalDriver, CpalDriverError> {
        if desired_latency_ms == 0 {
            return Err(format!("desired_latency_ms must be greater than 0").into());
        }

        let endpoint = get_default_endpoint().expect("Failed to get audio endpoint");

        let format = endpoint.get_supported_formats_list()
            .expect("Failed to get supported format list for endpoint")
            .find(|format| format.channels.len() == 2)
            .expect("Failed to find format with 2 channels");

        let buffer_frames = (sample_rate * desired_latency_ms / 1000 * 2) as usize;
        let ring_buffer = Arc::new(Mutex::new(RingBuffer {
            inner: vec![0; buffer_frames].into_iter().collect::<VecDeque<_>>(),
            samples_read: 0,
        }));

        let event_loop = EventLoop::new();

        let (mut voice, stream) = Voice::new(&endpoint, &format, &event_loop).expect("Failed to create voice");
        voice.play();

        let mut resampler = LinearResampler::new(sample_rate as _, format.samples_rate.0 as _);

        let read_ring_buffer = ring_buffer.clone();
        task::spawn(stream.for_each(move |output_buffer| {
            let mut read_ring_buffer = read_ring_buffer.lock().unwrap();

            match output_buffer {
                UnknownTypeBuffer::I16(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = resampler.next(&mut *read_ring_buffer);
                        }
                    }
                },
                UnknownTypeBuffer::U16(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = ((resampler.next(&mut *read_ring_buffer) as isize) + 32768) as u16;
                        }
                    }
                },
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = (resampler.next(&mut *read_ring_buffer) as f32) / 32768.0;
                        }
                    }
                },
            }

            Ok(())
        })).execute(Arc::new(CpalDriverExecutor));

        let join_handle = thread::spawn(move || {
            event_loop.run();
        });

        Ok(CpalDriver {
            ring_buffer: ring_buffer,
            sample_rate: sample_rate as _,

            _voice: voice,
            _join_handle: join_handle,
        })
    }

    pub fn sink(&self) -> Box<AudioBufferSink> {
        Box::new(CpalDriverBufferSink {
            ring_buffer: self.ring_buffer.clone(),
        })
    }

    pub fn time_source(&self) -> Box<TimeSource> {
        Box::new(CpalDriverTimeSource {
            ring_buffer: self.ring_buffer.clone(),
            sample_rate: self.sample_rate,
        })
    }
}

struct LinearResampler {
    from_sample_rate: usize,
    to_sample_rate: usize,

    current_from_frame: (i16, i16),
    next_from_frame: (i16, i16),
    from_fract_pos: usize,

    current_frame_channel_offset: usize,
}

impl LinearResampler {
    fn new(from_sample_rate: usize, to_sample_rate: usize) -> LinearResampler {
        let sample_rate_gcd = {
            fn gcd(a: usize, b: usize) -> usize {
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

    fn next(&mut self, input: &mut Iterator<Item = i16>) -> i16 {
        fn interpolate(a: i16, b: i16, num: usize, denom: usize) -> i16 {
            (((a as isize) * ((denom - num) as isize) + (b as isize) * (num as isize)) / (denom as isize)) as _
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
