use rodio::{Endpoint, Source, Sink, get_default_endpoint};

use audio_driver::*;

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::iter::Iterator;
use std::time::Duration;

pub type RodioDriverError = Cow<'static, str>;

struct RingBuffer {
    sample_rate: u32,

    len: usize,
    buf: Box<[i16]>,

    read_pos: usize,
    write_pos: usize,

    read_left: bool,
}

impl RingBuffer {
    fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<RingBuffer, RodioDriverError> {
        if desired_latency_ms == 0 {
            return Err(format!("desired_latency_ms must be greater than 0").into());
        }

        let len = (sample_rate * desired_latency_ms / 1000) as usize;
        let buf = vec![0; len * 2].into_boxed_slice();

        Ok(RingBuffer {
            sample_rate: sample_rate,

            len: len,
            buf: buf,

            read_pos: 0,
            write_pos: 0,

            read_left: true,
        })
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

struct RingBufferReader {
    sample_rate: u32,

    buffer: Arc<Mutex<RingBuffer>>,
}

impl Iterator for RingBufferReader {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        // TODO: Locking a mutex _every sample_ is _really bad_. REALLY. :)
        //  Unfortunately rodio doesn't appear to provide a low-level buffered
        //  interface, which is really unfortunate. Yet another reason why
        //  we probably have to do this audio abstraction stuff ourselves
        //  eventually.
        let mut buffer = self.buffer.lock().unwrap();

        let pos = buffer.read_pos * 2 + if buffer.read_left { 0 } else { 1 };
        let ret = Some(buffer.buf[pos]);

        buffer.read_left = !buffer.read_left;

        if buffer.read_left {
            buffer.read_pos += 1;
            if buffer.read_pos >= buffer.len {
                buffer.read_pos = 0;
            }
        }

        ret
    }
}

impl Source for RingBufferReader {
    fn get_current_frame_len(&self) -> Option<usize> {
        None
    }

    fn get_channels(&self) -> u16 {
        2
    }

    fn get_samples_rate(&self) -> u32 {
        self.sample_rate
    }

    fn get_total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct RodioDriver {
    len: usize,

    _endpoint: Endpoint,
    _sink: Sink,

    buffer: Arc<Mutex<RingBuffer>>,
}

impl RodioDriver {
    // Desired latency here will end up being latency _in addition to_ whatever the
    //  underlying rodio implementation gives us. Ideally this would be _total_
    //  latency, but given rodio's limited API this is what we're stuck with.
    pub fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<RodioDriver, RodioDriverError> {
        RingBuffer::new(sample_rate, desired_latency_ms).map(|buffer| {
            let sample_rate = buffer.sample_rate;
            let len = buffer.len;

            let buffer = Arc::new(Mutex::new(buffer));

            let endpoint = get_default_endpoint().unwrap();
            let sink = Sink::new(&endpoint);

            let reader = RingBufferReader {
                sample_rate: sample_rate,

                buffer: buffer.clone(),
            };

            sink.append(reader);

            RodioDriver {
                len: len,

                _endpoint: endpoint,
                _sink: sink,

                buffer: buffer,
            }
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn read_write_pos(&self) -> (usize, usize) {
        let buffer = self.buffer.lock().unwrap();

        (buffer.read_pos, buffer.write_pos)
    }
}

impl AudioDriver for RodioDriver {
    fn append_frame(&mut self, frame: (i16, i16)) {
        // TODO: See note for RingBufferReader.next
        self.buffer.lock().unwrap().append_frame(frame);
    }
}