use audio_driver::*;

use std::io::{self, Write, Seek, SeekFrom, BufWriter};
use std::fs::File;
use std::path::Path;

const NUM_CHANNELS: usize = 2;
const SAMPLE_RATE: usize = 41700;
const BITS_PER_SAMPLE: usize = 16;

pub struct WaveFileAudioDriver {
    writer: BufWriter<File>,
    num_frames: usize,
}

impl WaveFileAudioDriver {
    pub fn new<P: AsRef<Path>>(file_name: P) -> io::Result<WaveFileAudioDriver> {
        let file = File::create(file_name)?;
        let writer = BufWriter::new(file);

        let mut ret = WaveFileAudioDriver {
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
        ret.write_u32(SAMPLE_RATE as _)?;
        ret.write_u32((SAMPLE_RATE * NUM_CHANNELS * BITS_PER_SAMPLE / 8) as _)?;
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

impl Drop for WaveFileAudioDriver {
    fn drop(&mut self) {
        let data_chunk_size = self.num_frames * NUM_CHANNELS * BITS_PER_SAMPLE / 8;

        // Shouldn't be doing anything that can panic in drop,
        //  but this whole module is temp anyways, so we won't
        //  worry about this for now :)
        self.writer.seek(SeekFrom::Start(4)).unwrap();
        self.write_u32((data_chunk_size + 36) as _).unwrap(); // Data sub-chunk size
        self.writer.seek(SeekFrom::Start(40)).unwrap();
        self.write_u32(data_chunk_size as _).unwrap(); // Data size
    }
}

impl AudioDriver for WaveFileAudioDriver {
    fn output_frame(&mut self, (left, right): (i16, i16)) {
        self.write_u16(left as _).unwrap();
        self.write_u16(right as _).unwrap();
        self.num_frames += 1;
    }
}
