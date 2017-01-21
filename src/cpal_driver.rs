use cpal::{EventLoop, Voice, UnknownTypeBuffer, get_default_endpoint};

use futures::stream::Stream;
use futures::task::{self, Executor, Run};

use audio_driver::*;

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::iter::Iterator;
use std::thread::{self, JoinHandle};

pub type CpalDriverError = Cow<'static, str>;

struct RingBuffer {
    len: usize,
    buf: Box<[i16]>,

    read_pos: usize,
    write_pos: usize,

    read_left: bool,
}

impl RingBuffer {
    fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<RingBuffer, CpalDriverError> {
        if desired_latency_ms == 0 {
            return Err(format!("desired_latency_ms must be greater than 0").into());
        }

        let len = (sample_rate * desired_latency_ms / 1000) as usize;
        let buf = vec![0; len * 2].into_boxed_slice();

        Ok(RingBuffer {
            len: len,
            buf: buf,

            read_pos: 0,
            write_pos: 0,

            read_left: true,
        })
    }

    fn next(&mut self) -> i16 {
        let pos = self.read_pos * 2 + if self.read_left { 0 } else { 1 };
        let ret = self.buf[pos];

        self.read_left = !self.read_left;

        if self.read_left {
            self.read_pos += 1;
            if self.read_pos >= self.len {
                self.read_pos = 0;
            }
        }

        ret
    }

    fn append_frame(&mut self, (left, right): (i16, i16)) {
        let pos = self.write_pos * 2;
        self.buf[pos] = left;
        self.buf[pos + 1] = right;

        self.write_pos += 1;
        if self.write_pos >= self.len {
            self.write_pos = 0;
        }
    }
}

struct CpalDriverExecutor;

impl Executor for CpalDriverExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

pub struct CpalDriver {
    len: usize,

    ring_buffer: Arc<Mutex<RingBuffer>>,

    _voice: Voice,
    _join_handle: JoinHandle<()>,
}

impl CpalDriver {
    // TODO: Check that desired_latency gives us what we want here. It'll end up being whatever
    //  desired_latency plus whatever CPAL gives us currently, which may not be ideal (or it may
    //  be nothing).
    pub fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<CpalDriver, CpalDriverError> {
        let buffer = RingBuffer::new(sample_rate, desired_latency_ms)?;

        let len = buffer.len;

        let endpoint = get_default_endpoint().expect("Failed to get audio endpoint");
        let format = endpoint.get_supported_formats_list().expect("Failed to get supported format list for endpoint").next().expect("Failed to get endpoint format");
        if format.channels.len() != 2 {
            panic!("Endpoint format must be 2-channel");
        }

        let ring_buffer = Arc::new(Mutex::new(buffer));

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
                            *out = read_ring_buffer.next();
                        }
                    }
                },
                UnknownTypeBuffer::U16(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = ((read_ring_buffer.next() as isize) + 32768) as u16;
                        }
                    }
                },
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = (read_ring_buffer.next() as f32) / 32768.0;
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
            len: len,

            ring_buffer: ring_buffer,

            _voice: voice,
            _join_handle: join_handle,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn read_write_pos(&self) -> (usize, usize) {
        let ring_buffer = self.ring_buffer.lock().unwrap();

        (ring_buffer.read_pos, ring_buffer.write_pos)
    }
}

impl AudioDriver for CpalDriver {
    fn append_frame(&mut self, frame: (i16, i16)) {
        // TODO: Locking mutex each sample sucks; expose a buffered interface instead
        self.ring_buffer.lock().unwrap().append_frame(frame);
    }
}
