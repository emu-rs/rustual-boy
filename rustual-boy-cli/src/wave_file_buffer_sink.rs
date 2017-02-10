#![allow(dead_code)]

use rustual_boy_core::sinks::{AudioBufferSink, SinkRef};

use std::io::{self, Write, Seek, SeekFrom, BufWriter};
use std::fs::File;
use std::path::Path;

const NUM_CHANNELS: usize = 2;
const BITS_PER_SAMPLE: usize = 16;

pub struct WaveFileBufferSink {
    writer: BufWriter<File>,
    num_frames: usize,
}

impl WaveFileBufferSink {
    pub fn new<P: AsRef<Path>>(file_name: P, sample_rate: usize) -> io::Result<WaveFileBufferSink> {
        let file = File::create(file_name)?;
        let writer = BufWriter::new(file);

        let mut ret = WaveFileBufferSink {
            writer: writer,
            num_frames: 0,
        };

        // RIFF header
        ret.write_str("RIFF")?;
        ret.write_u32(0)?; // Data sub-chunk size; will be written properly later
        ret.write_str("WAVE")?;

        // Format sub-chunk
        ret.write_str("fmt ")?;
        ret.write_u32(16)?;
        ret.write_u16(1)?; // WAVE_FORMAT_PCM
        ret.write_u16(NUM_CHANNELS as _)?;
        ret.write_u32(sample_rate as _)?;
        ret.write_u32((sample_rate * NUM_CHANNELS * BITS_PER_SAMPLE / 8) as _)?;
        ret.write_u16((NUM_CHANNELS * BITS_PER_SAMPLE / 8) as _)?;
        ret.write_u16(BITS_PER_SAMPLE as _)?;

        // Data sub-chunk
        ret.write_str("data")?;
        ret.write_u32(0)?; // Data size; will be written properly later

        Ok(ret)
    }

    fn write_str(&mut self, value: &str) -> io::Result<()> {
        self.writer.write_all(value.as_bytes())?;

        Ok(())
    }

    fn write_u16(&mut self, value: u16) -> io::Result<()> {
        let buf = [value as u8, (value >> 8) as u8];

        self.writer.write_all(&buf)?;

        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        let buf = [value as u8, (value >> 8) as u8, (value >> 16) as u8, (value >> 24) as u8];

        self.writer.write_all(&buf)?;

        Ok(())
    }
}

impl Drop for WaveFileBufferSink {
    fn drop(&mut self) {
        let data_chunk_size = self.num_frames * NUM_CHANNELS * BITS_PER_SAMPLE / 8;

        let _ = self.writer.seek(SeekFrom::Start(4));
        let _ = self.write_u32((data_chunk_size + 36) as _); // Data sub-chunk size
        let _ = self.writer.seek(SeekFrom::Start(40));
        let _ = self.write_u32(data_chunk_size as _); // Data size
    }
}

impl SinkRef<[(i16, i16)]> for WaveFileBufferSink {
    fn append(&mut self, buffer: &[(i16, i16)]) {
        for &(left, right) in buffer {
            self.write_u16(left as _).unwrap();
            self.write_u16(right as _).unwrap();
            self.num_frames += 1;
        }
    }
}

impl AudioBufferSink for WaveFileBufferSink {
}
