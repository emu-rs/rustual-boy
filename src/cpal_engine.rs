use cpal::{EventLoop, Voice, UnknownTypeBuffer, get_default_endpoint};

use futures::stream::Stream;
use futures::task::{self, Executor, Run};

use audio_driver::*;

use std::borrow::Cow;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::iter::Iterator;
use std::thread::{self, JoinHandle};

pub type CpalEngineError = Cow<'static, str>;

pub struct RingBuffer {
    desired_len: usize,

    inner: VecDeque<i16>,
}

impl AudioDriver for RingBuffer {
    fn desired_frames(&self) -> usize {
        if self.inner.len() < self.desired_len {
            (self.desired_len - self.inner.len()) / 2
        } else {
            0
        }
    }

    fn append_frame(&mut self, (left, right): (i16, i16)) {
        self.inner.push_back(left);
        self.inner.push_back(right);
    }
}

struct CpalEngineExecutor;

impl Executor for CpalEngineExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

pub struct CpalEngine {
    ring_buffer: Arc<Mutex<RingBuffer>>,

    _voice: Voice,
    _join_handle: JoinHandle<()>,
}

impl CpalEngine {
    pub fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<CpalEngine, CpalEngineError> {
        if desired_latency_ms == 0 {
            return Err(format!("desired_latency_ms must be greater than 0").into());
        }

        let endpoint = get_default_endpoint().expect("Failed to get audio endpoint");
        let format = endpoint.get_supported_formats_list().expect("Failed to get supported format list for endpoint").next().expect("Failed to get endpoint format");
        if format.channels.len() != 2 {
            panic!("Endpoint format must be 2-channel");
        }

        let desired_len = (sample_rate * desired_latency_ms / 1000 * 2) as usize; // * 2 for stereo
        let ring_buffer = Arc::new(Mutex::new(RingBuffer {
            desired_len: desired_len,

            inner: vec![0; desired_len].into_iter().collect::<VecDeque<_>>(),
        }));

        let event_loop = EventLoop::new();

        let (mut voice, stream) = Voice::new(&endpoint, &format, &event_loop).expect("Failed to create voice");
        voice.play();

        let read_ring_buffer = ring_buffer.clone();
        task::spawn(stream.for_each(move |output_buffer| {
            let mut read_ring_buffer = read_ring_buffer.lock().unwrap();

            match output_buffer {
                UnknownTypeBuffer::I16(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = read_ring_buffer.inner.pop_front().unwrap_or(0);
                        }
                    }
                },
                UnknownTypeBuffer::U16(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = ((read_ring_buffer.inner.pop_front().unwrap_or(0) as isize) + 32768) as u16;
                        }
                    }
                },
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = (read_ring_buffer.inner.pop_front().unwrap_or(0) as f32) / 32768.0;
                        }
                    }
                },
            }

            Ok(())
        })).execute(Arc::new(CpalEngineExecutor));

        let join_handle = thread::spawn(move || {
            event_loop.run();
        });

        Ok(CpalEngine {
            ring_buffer: ring_buffer,

            _voice: voice,
            _join_handle: join_handle,
        })
    }

    pub fn driver(&self) -> Arc<Mutex<RingBuffer>> {
        self.ring_buffer.clone()
    }
}
