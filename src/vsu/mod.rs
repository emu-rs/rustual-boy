mod mem_map;

use audio_driver::*;
use self::mem_map::*;

const NUM_WAVE_TABLE_WORDS: usize = 32;
const NUM_WAVE_TABLES: usize = 5;
const TOTAL_WAVE_TABLE_SIZE: usize = NUM_WAVE_TABLE_WORDS * NUM_WAVE_TABLES;

const S_TO_NS: u64 = 1000000000;

// Assuming the sound unit runs off of a 20mhz clock, I did some math to check this sample rate,
//  and the closest sample rate with an integral clock rate division from 20mhz should actually be
//  41666.66 repeating (20 mhz / 480). This is most likely the real sample rate, but effectively
//  it won't make much practical difference.
const SAMPLE_RATE: u64 = 41700;
const SAMPLE_PERIOD_NS: u64 = S_TO_NS / SAMPLE_RATE;

const CPU_CYCLE_PERIOD_NS: u64 = 50;

const FREQUENCY_CLOCK_PERIOD_NS: u64 = S_TO_NS / 5000000;

#[derive(Default)]
struct Envelope {
    reg_data_reload: usize,
    reg_data_direction: bool,
    reg_data_step_interval: usize,

    reg_control_repeat: bool,
    reg_control_enable: bool,
}

impl Envelope {
    fn read_data_reg(&self) -> u8 {
        ((self.reg_data_reload as u8) << 4) |
        (if self.reg_data_direction { 1 } else { 0 } << 3) |
        (self.reg_data_step_interval as u8)
    }

    fn write_data_reg(&mut self, value: u8) {
        self.reg_data_reload = (value >> 4) as _;
        self.reg_data_direction = (value & 0x80) != 0;
        self.reg_data_step_interval = (value & 0x07) as _;
    }

    fn read_control_reg(&self) -> u8 {
        (if self.reg_control_repeat { 1 } else { 0 } << 1) |
        (if self.reg_control_enable { 1 } else { 0 })
    }

    fn write_control_reg(&mut self, value: u8) {
        self.reg_control_repeat = (value & 0x02) != 0;
        self.reg_control_enable = (value & 0x01) != 0;
    }
}

#[derive(Default)]
struct StandardVoice {
    reg_play_control_enable: bool,
    reg_play_control_use_duration: bool,
    reg_play_control_duration: usize,

    reg_volume_left: usize,
    reg_volume_right: usize,

    reg_frequency_low: usize,

    reg_frequency_high: usize,

    envelope: Envelope,

    reg_pcm_wave: usize,

    frequency_clock_counter: u64,
    frequency_counter: usize,
    phase: usize,
}

impl StandardVoice {
    fn read_play_control_reg(&self) -> u8 {
        (if self.reg_play_control_enable { 1 } else { 0 } << 7) |
        (if self.reg_play_control_use_duration { 1 } else { 0 } << 5) |
        (self.reg_play_control_duration as u8)
    }

    fn write_play_control_reg(&mut self, value: u8) {
        self.reg_play_control_enable = (value & 0x80) != 0;
        self.reg_play_control_use_duration = (value & 0x20) != 0;
        self.reg_play_control_duration = (value & 0xff) as _;

        if self.reg_play_control_enable {
            self.frequency_clock_counter = 0;
            self.frequency_counter = 0;
            self.phase = 0;
        }
    }

    fn read_volume_reg(&self) -> u8 {
        ((self.reg_volume_left as u8) << 4) |
        (self.reg_volume_right as u8)
    }

    fn write_volume_reg(&mut self, value: u8) {
        self.reg_volume_left = (value >> 4) as _;
        self.reg_volume_right = (value & 0x0f) as _;
    }

    fn read_frequency_low_reg(&self) -> u8 {
        self.reg_frequency_low as _
    }

    fn write_frequency_low_reg(&mut self, value: u8) {
        self.reg_frequency_low = value as _;
    }

    fn read_frequency_high_reg(&self) -> u8 {
        self.reg_frequency_high as _
    }

    fn write_frequency_high_reg(&mut self, value: u8) {
        self.reg_frequency_high = (value & 0x07) as _;
    }

    fn read_envelope_data_reg(&self) -> u8 {
        self.envelope.read_data_reg()
    }

    fn write_envelope_data_reg(&mut self, value: u8) {
        self.envelope.write_data_reg(value);
    }

    fn read_envelope_control_reg(&self) -> u8 {
        self.envelope.read_control_reg()
    }

    fn write_envelope_control_reg(&mut self, value: u8) {
        self.envelope.write_control_reg(value);
    }

    fn read_pcm_wave_reg(&self) -> u8 {
        self.reg_pcm_wave as _
    }

    fn write_pcm_wave_reg(&mut self, value: u8) {
        self.reg_pcm_wave = (value & 0x07) as _;
    }

    fn cycle(&mut self) {
        self.frequency_clock_counter += CPU_CYCLE_PERIOD_NS;
        if self.frequency_clock_counter >= FREQUENCY_CLOCK_PERIOD_NS {
            self.frequency_clock_counter -= FREQUENCY_CLOCK_PERIOD_NS;

            self.frequency_counter += 1;
            if self.frequency_counter >= 2048 - ((self.reg_frequency_high << 8) | self.reg_frequency_low) {
                self.frequency_counter = 0;

                self.frequency_clock();
            }
        }
    }

    fn frequency_clock(&mut self) {
        self.phase = (self.phase + 1) & (NUM_WAVE_TABLE_WORDS - 1);
    }

    fn sample(&self, wave_tables: &[u8]) -> (usize, usize) {
        let wave_level = wave_tables[self.reg_pcm_wave * NUM_WAVE_TABLE_WORDS + self.phase] as usize;

        self.mix_sample(wave_level)
    }

    fn mix_sample(&self, sound_data: usize) -> (usize, usize) {
        if self.reg_play_control_enable {
            let envelope_level = 0x0f;//if self.envelope.reg_control_enable { 0x08 } else { 0x00 };

            let left_level = if self.reg_volume_left == 0 || envelope_level == 0 {
                0
            } else {
                (self.reg_volume_left * envelope_level) + 1
            };
            let right_level = if self.reg_volume_right == 0 || envelope_level == 0 {
                0
            } else {
                (self.reg_volume_right * envelope_level) + 1
            };

            let output_left = (sound_data * left_level) >> 1;
            let output_right = (sound_data * right_level) >> 1;

            (output_left, output_right)
        } else {
            (0, 0)
        }
    }
}

pub struct Vsu {
    wave_tables: Box<[u8]>,

    voice1: StandardVoice,
    voice2: StandardVoice,
    voice3: StandardVoice,
    voice4: StandardVoice,
    voice5: StandardVoice,

    reg_sound_disable: bool,

    sample_clock_counter: u64,
}

impl Vsu {
    pub fn new() -> Vsu {
        Vsu {
            wave_tables: vec![0; TOTAL_WAVE_TABLE_SIZE].into_boxed_slice(),

            voice1: StandardVoice::default(),
            voice2: StandardVoice::default(),
            voice3: StandardVoice::default(),
            voice4: StandardVoice::default(),
            voice5: StandardVoice::default(),

            reg_sound_disable: false,

            sample_clock_counter: 0,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        match addr {
            PCM_WAVE_TABLE_0_START ... PCM_WAVE_TABLE_0_END => self.wave_tables[((addr - PCM_WAVE_TABLE_0_START) / 4 + 0x00) as usize],
            PCM_WAVE_TABLE_1_START ... PCM_WAVE_TABLE_1_END => self.wave_tables[((addr - PCM_WAVE_TABLE_1_START) / 4 + 0x20) as usize],
            PCM_WAVE_TABLE_2_START ... PCM_WAVE_TABLE_2_END => self.wave_tables[((addr - PCM_WAVE_TABLE_2_START) / 4 + 0x40) as usize],
            PCM_WAVE_TABLE_3_START ... PCM_WAVE_TABLE_3_END => self.wave_tables[((addr - PCM_WAVE_TABLE_3_START) / 4 + 0x60) as usize],
            PCM_WAVE_TABLE_4_START ... PCM_WAVE_TABLE_4_END => self.wave_tables[((addr - PCM_WAVE_TABLE_4_START) / 4 + 0x80) as usize],
            VOICE_1_PLAY_CONTROL => self.voice1.read_play_control_reg(),
            VOICE_1_VOLUME => self.voice1.read_volume_reg(),
            VOICE_1_FREQUENCY_LOW => self.voice1.read_frequency_low_reg(),
            VOICE_1_FREQUENCY_HIGH => self.voice1.read_frequency_high_reg(),
            VOICE_1_ENVELOPE_DATA => self.voice1.read_envelope_data_reg(),
            VOICE_1_ENVELOPE_CONTROL => self.voice1.read_envelope_control_reg(),
            VOICE_1_PCM_WAVE => self.voice1.read_pcm_wave_reg(),
            VOICE_2_PLAY_CONTROL => self.voice2.read_play_control_reg(),
            VOICE_2_VOLUME => self.voice2.read_volume_reg(),
            VOICE_2_FREQUENCY_LOW => self.voice2.read_frequency_low_reg(),
            VOICE_2_FREQUENCY_HIGH => self.voice2.read_frequency_high_reg(),
            VOICE_2_ENVELOPE_DATA => self.voice2.read_envelope_data_reg(),
            VOICE_2_ENVELOPE_CONTROL => self.voice2.read_envelope_control_reg(),
            VOICE_2_PCM_WAVE => self.voice2.read_pcm_wave_reg(),
            VOICE_3_PLAY_CONTROL => self.voice3.read_play_control_reg(),
            VOICE_3_VOLUME => self.voice3.read_volume_reg(),
            VOICE_3_FREQUENCY_LOW => self.voice3.read_frequency_low_reg(),
            VOICE_3_FREQUENCY_HIGH => self.voice3.read_frequency_high_reg(),
            VOICE_3_ENVELOPE_DATA => self.voice3.read_envelope_data_reg(),
            VOICE_3_ENVELOPE_CONTROL => self.voice3.read_envelope_control_reg(),
            VOICE_3_PCM_WAVE => self.voice3.read_pcm_wave_reg(),
            VOICE_4_PLAY_CONTROL => self.voice4.read_play_control_reg(),
            VOICE_4_VOLUME => self.voice4.read_volume_reg(),
            VOICE_4_FREQUENCY_LOW => self.voice4.read_frequency_low_reg(),
            VOICE_4_FREQUENCY_HIGH => self.voice4.read_frequency_high_reg(),
            VOICE_4_ENVELOPE_DATA => self.voice4.read_envelope_data_reg(),
            VOICE_4_ENVELOPE_CONTROL => self.voice4.read_envelope_control_reg(),
            VOICE_4_PCM_WAVE => self.voice4.read_pcm_wave_reg(),
            VOICE_5_PLAY_CONTROL => self.voice5.read_play_control_reg(),
            VOICE_5_VOLUME => self.voice5.read_volume_reg(),
            VOICE_5_FREQUENCY_LOW => self.voice5.read_frequency_low_reg(),
            VOICE_5_FREQUENCY_HIGH => self.voice5.read_frequency_high_reg(),
            VOICE_5_ENVELOPE_DATA => self.voice5.read_envelope_data_reg(),
            VOICE_5_ENVELOPE_CONTROL => self.voice5.read_envelope_control_reg(),
            VOICE_5_PCM_WAVE => self.voice5.read_pcm_wave_reg(),
            SOUND_DISABLE_REG => if self.reg_sound_disable { 1 } else { 0 },
            _ => {
                logln!("VSU read byte not yet implemented (addr: 0x{:08x})", addr);
                0
            }
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match addr {
            PCM_WAVE_TABLE_0_START ... PCM_WAVE_TABLE_0_END => self.wave_tables[((addr - PCM_WAVE_TABLE_0_START) / 4 + 0x00) as usize] = value & 0x3f,
            PCM_WAVE_TABLE_1_START ... PCM_WAVE_TABLE_1_END => self.wave_tables[((addr - PCM_WAVE_TABLE_1_START) / 4 + 0x20) as usize] = value & 0x3f,
            PCM_WAVE_TABLE_2_START ... PCM_WAVE_TABLE_2_END => self.wave_tables[((addr - PCM_WAVE_TABLE_2_START) / 4 + 0x40) as usize] = value & 0x3f,
            PCM_WAVE_TABLE_3_START ... PCM_WAVE_TABLE_3_END => self.wave_tables[((addr - PCM_WAVE_TABLE_3_START) / 4 + 0x60) as usize] = value & 0x3f,
            PCM_WAVE_TABLE_4_START ... PCM_WAVE_TABLE_4_END => self.wave_tables[((addr - PCM_WAVE_TABLE_4_START) / 4 + 0x80) as usize] = value & 0x3f,
            VOICE_1_PLAY_CONTROL => self.voice1.write_play_control_reg(value),
            VOICE_1_VOLUME => self.voice1.write_volume_reg(value),
            VOICE_1_FREQUENCY_LOW => self.voice1.write_frequency_low_reg(value),
            VOICE_1_FREQUENCY_HIGH => self.voice1.write_frequency_high_reg(value),
            VOICE_1_ENVELOPE_DATA => self.voice1.write_envelope_data_reg(value),
            VOICE_1_ENVELOPE_CONTROL => self.voice1.write_envelope_control_reg(value),
            VOICE_1_PCM_WAVE => self.voice1.write_pcm_wave_reg(value),
            VOICE_2_PLAY_CONTROL => self.voice2.write_play_control_reg(value),
            VOICE_2_VOLUME => self.voice2.write_volume_reg(value),
            VOICE_2_FREQUENCY_LOW => self.voice2.write_frequency_low_reg(value),
            VOICE_2_FREQUENCY_HIGH => self.voice2.write_frequency_high_reg(value),
            VOICE_2_ENVELOPE_DATA => self.voice2.write_envelope_data_reg(value),
            VOICE_2_ENVELOPE_CONTROL => self.voice2.write_envelope_control_reg(value),
            VOICE_2_PCM_WAVE => self.voice2.write_pcm_wave_reg(value),
            VOICE_3_PLAY_CONTROL => self.voice3.write_play_control_reg(value),
            VOICE_3_VOLUME => self.voice3.write_volume_reg(value),
            VOICE_3_FREQUENCY_LOW => self.voice3.write_frequency_low_reg(value),
            VOICE_3_FREQUENCY_HIGH => self.voice3.write_frequency_high_reg(value),
            VOICE_3_ENVELOPE_DATA => self.voice3.write_envelope_data_reg(value),
            VOICE_3_ENVELOPE_CONTROL => self.voice3.write_envelope_control_reg(value),
            VOICE_3_PCM_WAVE => self.voice3.write_pcm_wave_reg(value),
            VOICE_4_PLAY_CONTROL => self.voice4.write_play_control_reg(value),
            VOICE_4_VOLUME => self.voice4.write_volume_reg(value),
            VOICE_4_FREQUENCY_LOW => self.voice4.write_frequency_low_reg(value),
            VOICE_4_FREQUENCY_HIGH => self.voice4.write_frequency_high_reg(value),
            VOICE_4_ENVELOPE_DATA => self.voice4.write_envelope_data_reg(value),
            VOICE_4_ENVELOPE_CONTROL => self.voice4.write_envelope_control_reg(value),
            VOICE_4_PCM_WAVE => self.voice4.write_pcm_wave_reg(value),
            VOICE_5_PLAY_CONTROL => self.voice5.write_play_control_reg(value),
            VOICE_5_VOLUME => self.voice5.write_volume_reg(value),
            VOICE_5_FREQUENCY_LOW => self.voice5.write_frequency_low_reg(value),
            VOICE_5_FREQUENCY_HIGH => self.voice5.write_frequency_high_reg(value),
            VOICE_5_ENVELOPE_DATA => self.voice5.write_envelope_data_reg(value),
            VOICE_5_ENVELOPE_CONTROL => self.voice5.write_envelope_control_reg(value),
            VOICE_5_PCM_WAVE => self.voice5.write_pcm_wave_reg(value),
            SOUND_DISABLE_REG => {
                // This might actually be a strobe register
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
        for _ in 0..cycles {
            self.voice1.cycle();
            self.voice2.cycle();
            self.voice3.cycle();
            self.voice4.cycle();
            self.voice5.cycle();

            self.sample_clock_counter += CPU_CYCLE_PERIOD_NS;
            if self.sample_clock_counter >= SAMPLE_PERIOD_NS {
                self.sample_clock_counter -= SAMPLE_PERIOD_NS;

                /*if self.reg_sound_disable {
                    audio_driver.output_frame((0, 0));
                } else {*/
                    let mut voice_acc_left = 0;
                    let mut voice_acc_right = 0;

                    let (voice_left, voice_right) = self.voice1.sample(&self.wave_tables);
                    voice_acc_left += voice_left;
                    voice_acc_right += voice_right;
                    let (voice_left, voice_right) = self.voice2.sample(&self.wave_tables);
                    voice_acc_left += voice_left;
                    voice_acc_right += voice_right;
                    let (voice_left, voice_right) = self.voice3.sample(&self.wave_tables);
                    voice_acc_left += voice_left;
                    voice_acc_right += voice_right;
                    let (voice_left, voice_right) = self.voice4.sample(&self.wave_tables);
                    voice_acc_left += voice_left;
                    voice_acc_right += voice_right;
                    let (voice_left, voice_right) = self.voice5.sample(&self.wave_tables);
                    voice_acc_left += voice_left;
                    voice_acc_right += voice_right;

                    let output_left = (voice_acc_left & 0xfff8) as i16;
                    let output_right = (voice_acc_right & 0xfff8) as i16;

                    audio_driver.output_frame((output_left, output_right));
                //}
            }
        }
    }
}