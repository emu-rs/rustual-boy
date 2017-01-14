mod mem_map;

use audio_driver::*;
use self::mem_map::*;

const NUM_WAVE_TABLE_WORDS: usize = 32;
const NUM_WAVE_TABLES: usize = 5;
const TOTAL_WAVE_TABLE_SIZE: usize = NUM_WAVE_TABLE_WORDS * NUM_WAVE_TABLES;

#[derive(Default, Clone)]
struct Voice {
    reg_play_control_enable: bool,
    reg_play_control_use_duration: bool,
    reg_play_control_duration: usize,

    reg_volume_left: usize,
    reg_volume_right: usize,

    reg_frequency_low: usize,

    reg_frequency_high: usize,

    reg_envelope_data_reload: usize,
    reg_envelope_data_direction: bool,
    reg_envelope_data_step_interval: usize,

    reg_envelope_control_repeat: bool,
    reg_envelope_control_enable: bool,

    reg_pcm_wave: usize,
}

impl Voice {
    fn read_play_control_reg(&self) -> u8 {
        (if self.reg_play_control_enable { 1 } else { 0 } << 7) |
        (if self.reg_play_control_use_duration { 1 } else { 0 } << 5) |
        (self.reg_play_control_duration as u8)
    }

    fn read_volume_reg(&self) -> u8 {
        ((self.reg_volume_left as u8) << 4) |
        (self.reg_volume_right as u8)
    }

    fn read_frequency_low_reg(&self) -> u8 {
        self.reg_frequency_low as _
    }

    fn read_frequency_high_reg(&self) -> u8 {
        self.reg_frequency_high as _
    }

    fn read_envelope_data_reg(&self) -> u8 {
        ((self.reg_envelope_data_reload as u8) << 4) |
        (if self.reg_envelope_data_direction { 1 } else { 0 } << 3) |
        (self.reg_envelope_data_step_interval as u8)
    }

    fn read_envelope_control_reg(&self) -> u8 {
        (if self.reg_envelope_control_repeat { 1 } else { 0 } << 1) |
        (if self.reg_envelope_control_enable { 1 } else { 0 })
    }

    fn read_pcm_wave_reg(&self) -> u8 {
        self.reg_pcm_wave as _
    }
}

pub struct Vsu {
    wave_tables: Box<[u8]>,

    voices: Box<[Voice]>,

    reg_sound_disable: bool,
}

impl Vsu {
    pub fn new() -> Vsu {
        Vsu {
            wave_tables: vec![0; TOTAL_WAVE_TABLE_SIZE].into_boxed_slice(),

            voices: vec![Voice::default(); 4].into_boxed_slice(),

            reg_sound_disable: false,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        match addr {
            PCM_WAVE_TABLE_0_START ... PCM_WAVE_TABLE_0_END => self.wave_tables[((addr - PCM_WAVE_TABLE_0_START) / 4) as usize],
            PCM_WAVE_TABLE_1_START ... PCM_WAVE_TABLE_1_END => self.wave_tables[((addr - PCM_WAVE_TABLE_1_START) / 4) as usize],
            PCM_WAVE_TABLE_2_START ... PCM_WAVE_TABLE_2_END => self.wave_tables[((addr - PCM_WAVE_TABLE_2_START) / 4) as usize],
            PCM_WAVE_TABLE_3_START ... PCM_WAVE_TABLE_3_END => self.wave_tables[((addr - PCM_WAVE_TABLE_3_START) / 4) as usize],
            PCM_WAVE_TABLE_4_START ... PCM_WAVE_TABLE_4_END => self.wave_tables[((addr - PCM_WAVE_TABLE_4_START) / 4) as usize],
            VOICE_1_PLAY_CONTROL => self.voices[0].read_play_control_reg(),
            VOICE_1_VOLUME => self.voices[0].read_volume_reg(),
            VOICE_1_FREQUENCY_LOW => self.voices[0].read_frequency_low_reg(),
            VOICE_1_FREQUENCY_HIGH => self.voices[0].read_frequency_high_reg(),
            VOICE_1_ENVELOPE_DATA => self.voices[0].read_envelope_data_reg(),
            VOICE_1_ENVELOPE_CONTROL => self.voices[0].read_envelope_control_reg(),
            VOICE_1_PCM_WAVE => self.voices[0].read_pcm_wave_reg(),
            VOICE_2_PLAY_CONTROL => self.voices[1].read_play_control_reg(),
            VOICE_2_VOLUME => self.voices[1].read_volume_reg(),
            VOICE_2_FREQUENCY_LOW => self.voices[1].read_frequency_low_reg(),
            VOICE_2_FREQUENCY_HIGH => self.voices[1].read_frequency_high_reg(),
            VOICE_2_ENVELOPE_DATA => self.voices[1].read_envelope_data_reg(),
            VOICE_2_ENVELOPE_CONTROL => self.voices[1].read_envelope_control_reg(),
            VOICE_2_PCM_WAVE => self.voices[1].read_pcm_wave_reg(),
            VOICE_3_PLAY_CONTROL => self.voices[2].read_play_control_reg(),
            VOICE_3_VOLUME => self.voices[2].read_volume_reg(),
            VOICE_3_FREQUENCY_LOW => self.voices[2].read_frequency_low_reg(),
            VOICE_3_FREQUENCY_HIGH => self.voices[2].read_frequency_high_reg(),
            VOICE_3_ENVELOPE_DATA => self.voices[2].read_envelope_data_reg(),
            VOICE_3_ENVELOPE_CONTROL => self.voices[2].read_envelope_control_reg(),
            VOICE_3_PCM_WAVE => self.voices[2].read_pcm_wave_reg(),
            VOICE_4_PLAY_CONTROL => self.voices[3].read_play_control_reg(),
            VOICE_4_VOLUME => self.voices[3].read_volume_reg(),
            VOICE_4_FREQUENCY_LOW => self.voices[3].read_frequency_low_reg(),
            VOICE_4_FREQUENCY_HIGH => self.voices[3].read_frequency_high_reg(),
            VOICE_4_ENVELOPE_DATA => self.voices[3].read_envelope_data_reg(),
            VOICE_4_ENVELOPE_CONTROL => self.voices[3].read_envelope_control_reg(),
            VOICE_4_PCM_WAVE => self.voices[3].read_pcm_wave_reg(),
            SOUND_DISABLE_REG => if self.reg_sound_disable { 1 } else { 0 },
            _ => {
                logln!("VSU read byte not yet implemented (addr: 0x{:08x})", addr);
                0
            }
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match addr {
            PCM_WAVE_TABLE_0_START ... PCM_WAVE_TABLE_0_END => self.wave_tables[((addr - PCM_WAVE_TABLE_0_START) / 4) as usize] = value,
            PCM_WAVE_TABLE_1_START ... PCM_WAVE_TABLE_1_END => self.wave_tables[((addr - PCM_WAVE_TABLE_1_START) / 4) as usize] = value,
            PCM_WAVE_TABLE_2_START ... PCM_WAVE_TABLE_2_END => self.wave_tables[((addr - PCM_WAVE_TABLE_2_START) / 4) as usize] = value,
            PCM_WAVE_TABLE_3_START ... PCM_WAVE_TABLE_3_END => self.wave_tables[((addr - PCM_WAVE_TABLE_3_START) / 4) as usize] = value,
            PCM_WAVE_TABLE_4_START ... PCM_WAVE_TABLE_4_END => self.wave_tables[((addr - PCM_WAVE_TABLE_4_START) / 4) as usize] = value,
            SOUND_DISABLE_REG => {
                self.reg_sound_disable = (value & 0x01) != 0;
            }
            _ => logln!("VSU write byte not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value)
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        self.read_byte(addr) as _
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        self.write_byte(addr, value as _);
    }

    pub fn cycles(&mut self, cycles: usize, audio_driver: &mut AudioDriver) {
        // TODO
    }
}