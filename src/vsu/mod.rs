mod mem_map;

use audio_frame_sink::*;
use self::mem_map::*;

// Docs claim the sample rate is 41.7khz, but my calculations indicate it should be 41666.66hz repeating
//  (see SAMPLE_CLOCK_PERIOD calculation below), so we take the nearest whole-number sample rate to that.
//  Note that the documentation rounds values in a lot of places, so that's probably what happened here.
pub const SAMPLE_RATE: usize = 41667;

// 20mhz / 41.7khz = ~480 clocks
const SAMPLE_CLOCK_PERIOD: usize = 480;

// 20mhz / 260.4hz = ~76805 clocks
const DURATION_CLOCK_PERIOD: usize = 76805;

// 20mhz / 65.1hz = ~307218 clocks
const ENVELOPE_CLOCK_PERIOD: usize = 307218;

// 20mhz / 5mhz = 4 clocks
const FREQUENCY_CLOCK_PERIOD: usize = 4;

// 20mhz / 500khz = 40 clocks
const NOISE_CLOCK_PERIOD: usize = 40;

const NUM_WAVE_TABLE_WORDS: usize = 32;
const NUM_WAVE_TABLES: usize = 5;
const TOTAL_WAVE_TABLE_SIZE: usize = NUM_WAVE_TABLE_WORDS * NUM_WAVE_TABLES;

#[derive(Default)]
struct PlayControlReg {
    enable: bool,
    use_duration: bool,
    duration: usize,

    duration_counter: usize,
}

impl PlayControlReg {
    fn write(&mut self, value: u8) {
        self.enable = (value & 0x80) != 0;
        self.use_duration = (value & 0x20) != 0;
        self.duration = (value & 0x1f) as _;

        if self.use_duration {
            self.duration_counter = 0;
        }
    }

    fn duration_clock(&mut self) {
        if self.enable && self.use_duration {
            self.duration_counter += 1;
            if self.duration_counter > self.duration {
                self.enable = false;
            }
        }
    }
}

#[derive(Default)]
struct VolumeReg {
    left: usize,
    right: usize,
}

impl VolumeReg {
    fn write(&mut self, value: u8) {
        self.left = (value >> 4) as _;
        self.right = (value & 0x0f) as _;
    }
}

#[derive(Default)]
struct Envelope {
    reg_data_reload: usize,
    reg_data_direction: bool,
    reg_data_step_interval: usize,

    reg_control_repeat: bool,
    reg_control_enable: bool,

    level: usize,

    envelope_counter: usize,
}

impl Envelope {
    fn write_data_reg(&mut self, value: u8) {
        self.reg_data_reload = (value >> 4) as _;
        self.reg_data_direction = (value & 0x08) != 0;
        self.reg_data_step_interval = (value & 0x07) as _;

        self.level = self.reg_data_reload;
    }

    fn write_control_reg(&mut self, value: u8) {
        self.reg_control_repeat = (value & 0x02) != 0;
        self.reg_control_enable = (value & 0x01) != 0;
    }

    fn level(&self) -> usize {
        self.level
    }

    fn envelope_clock(&mut self) {
        if self.reg_control_enable {
            self.envelope_counter += 1;
            if self.envelope_counter > self.reg_data_step_interval {
                self.envelope_counter = 0;

                if self.reg_data_direction && self.level < 15 {
                    self.level += 1;
                } else if !self.reg_data_direction && self.level > 0 {
                    self.level -= 1;
                } else if self.reg_control_repeat {
                    self.level = self.reg_data_reload;
                }
            }
        }
    }
}

trait Voice {
    fn reg_play_control(&self) -> &PlayControlReg;
    fn reg_volume(&self) -> &VolumeReg;
    fn envelope(&self) -> &Envelope;
}

#[derive(Default)]
struct StandardVoice {
    reg_play_control: PlayControlReg,

    reg_volume: VolumeReg,

    reg_frequency_low: usize,
    reg_frequency_high: usize,

    envelope: Envelope,

    reg_pcm_wave: usize,

    frequency_counter: usize,
    phase: usize,
}

impl StandardVoice {
    fn write_play_control_reg(&mut self, value: u8) {
        self.reg_play_control.write(value);

        if self.reg_play_control.enable {
            self.envelope.envelope_counter = 0;

            self.frequency_counter = 0;
            self.phase = 0;
        }
    }

    fn write_volume_reg(&mut self, value: u8) {
        self.reg_volume.write(value);
    }

    fn write_frequency_low_reg(&mut self, value: u8) {
        self.reg_frequency_low = value as _;
    }

    fn write_frequency_high_reg(&mut self, value: u8) {
        self.reg_frequency_high = (value & 0x07) as _;
    }

    fn write_envelope_data_reg(&mut self, value: u8) {
        self.envelope.write_data_reg(value);
    }

    fn write_envelope_control_reg(&mut self, value: u8) {
        self.envelope.write_control_reg(value);
    }

    fn write_pcm_wave_reg(&mut self, value: u8) {
        self.reg_pcm_wave = (value & 0x07) as _;
    }

    fn frequency_clock(&mut self) {
        self.frequency_counter += 1;
        if self.frequency_counter >= 2048 - ((self.reg_frequency_high << 8) | self.reg_frequency_low) {
            self.frequency_counter = 0;

            self.phase = (self.phase + 1) & (NUM_WAVE_TABLE_WORDS - 1);
        }
    }

    fn output(&self, wave_tables: &[u8]) -> usize {
        if self.reg_pcm_wave > 4 {
            return 0;
        }

        wave_tables[self.reg_pcm_wave * NUM_WAVE_TABLE_WORDS + self.phase] as _
    }
}

impl Voice for StandardVoice {
    fn reg_play_control(&self) -> &PlayControlReg {
        &self.reg_play_control
    }

    fn reg_volume(&self) -> &VolumeReg {
        &self.reg_volume
    }

    fn envelope(&self) -> &Envelope {
        &self.envelope
    }
}

#[derive(Default)]
struct NoiseVoice {
    reg_play_control: PlayControlReg,

    reg_volume: VolumeReg,

    reg_frequency_low: usize,
    reg_frequency_high: usize,

    envelope: Envelope,

    reg_noise_control: usize,

    frequency_counter: usize,
    shift: usize,
    output: usize,
}

impl NoiseVoice {
    fn write_play_control_reg(&mut self, value: u8) {
        self.reg_play_control.write(value);

        if self.reg_play_control.enable {
            self.envelope.envelope_counter = 0;

            self.frequency_counter = 0;
            self.shift = 0x7fff;
        }
    }

    fn write_volume_reg(&mut self, value: u8) {
        self.reg_volume.write(value);
    }

    fn write_frequency_low_reg(&mut self, value: u8) {
        self.reg_frequency_low = value as _;
    }

    fn write_frequency_high_reg(&mut self, value: u8) {
        self.reg_frequency_high = (value & 0x07) as _;
    }

    fn write_envelope_data_reg(&mut self, value: u8) {
        self.envelope.write_data_reg(value);
    }

    fn write_envelope_noise_control_reg(&mut self, value: u8) {
        self.reg_noise_control = ((value >> 4) & 0x07) as _;
        self.envelope.write_control_reg(value);
    }

    fn noise_clock(&mut self) {
        self.frequency_counter += 1;
        if self.frequency_counter >= 2048 - ((self.reg_frequency_high << 8) | self.reg_frequency_low) {
            self.frequency_counter = 0;

            let lhs = self.shift >> 7;

            let rhs_bit_index = match self.reg_noise_control {
                0 => 14,
                1 => 10,
                2 => 13,
                3 => 4,
                4 => 8,
                5 => 6,
                6 => 9,
                _ => 11
            };
            let rhs = self.shift >> rhs_bit_index;

            let xor_bit = (lhs ^ rhs) & 0x01;

            self.shift = ((self.shift << 1) | xor_bit) & 0x7fff;

            let output_bit = (!xor_bit) & 0x01;
            self.output = match output_bit {
                0 => 0,
                _ => 0x3f
            };
        }
    }

    fn output(&self) -> usize {
        self.output
    }
}

impl Voice for NoiseVoice {
    fn reg_play_control(&self) -> &PlayControlReg {
        &self.reg_play_control
    }

    fn reg_volume(&self) -> &VolumeReg {
        &self.reg_volume
    }

    fn envelope(&self) -> &Envelope {
        &self.envelope
    }
}

pub struct Vsu {
    wave_tables: Box<[u8]>,

    voice1: StandardVoice,
    voice2: StandardVoice,
    voice3: StandardVoice,
    voice4: StandardVoice,
    voice5: StandardVoice,
    voice6: NoiseVoice,

    duration_clock_counter: usize,
    envelope_clock_counter: usize,
    frequency_clock_counter: usize,
    noise_clock_counter: usize,
    sample_clock_counter: usize,
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
            voice6: NoiseVoice::default(),

            duration_clock_counter: 0,
            envelope_clock_counter: 0,
            frequency_clock_counter: 0,
            noise_clock_counter: 0,
            sample_clock_counter: 0,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        logln!("WARNING: Attempted read byte from VSU (addr: 0x{:08x})", addr);

        0
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match addr {
            PCM_WAVE_TABLE_0_START ... PCM_WAVE_TABLE_0_END => {
                if !self.are_channels_active() {
                    self.wave_tables[((addr - PCM_WAVE_TABLE_0_START) / 4 + 0x00) as usize] = value & 0x3f;
                }
            }
            PCM_WAVE_TABLE_1_START ... PCM_WAVE_TABLE_1_END => {
                if !self.are_channels_active() {
                    self.wave_tables[((addr - PCM_WAVE_TABLE_1_START) / 4 + 0x20) as usize] = value & 0x3f;
                }
            }
            PCM_WAVE_TABLE_2_START ... PCM_WAVE_TABLE_2_END => {
                if !self.are_channels_active() {
                    self.wave_tables[((addr - PCM_WAVE_TABLE_2_START) / 4 + 0x40) as usize] = value & 0x3f;
                }
            }
            PCM_WAVE_TABLE_3_START ... PCM_WAVE_TABLE_3_END => {
                if !self.are_channels_active() {
                    self.wave_tables[((addr - PCM_WAVE_TABLE_3_START) / 4 + 0x60) as usize] = value & 0x3f;
                }
            }
            PCM_WAVE_TABLE_4_START ... PCM_WAVE_TABLE_4_END => {
                if !self.are_channels_active() {
                    self.wave_tables[((addr - PCM_WAVE_TABLE_4_START) / 4 + 0x80) as usize] = value & 0x3f;
                }
            }
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
            VOICE_6_PLAY_CONTROL => self.voice6.write_play_control_reg(value),
            VOICE_6_VOLUME => self.voice6.write_volume_reg(value),
            VOICE_6_FREQUENCY_LOW => self.voice6.write_frequency_low_reg(value),
            VOICE_6_FREQUENCY_HIGH => self.voice6.write_frequency_high_reg(value),
            VOICE_6_ENVELOPE_DATA => self.voice6.write_envelope_data_reg(value),
            VOICE_6_ENVELOPE_NOISE_CONTROL => self.voice6.write_envelope_noise_control_reg(value),
            SOUND_DISABLE_REG => {
                if (value & 0x01) != 0 {
                    self.voice1.reg_play_control.enable = false;
                    self.voice2.reg_play_control.enable = false;
                    self.voice3.reg_play_control.enable = false;
                    self.voice4.reg_play_control.enable = false;
                    self.voice5.reg_play_control.enable = false;
                    self.voice6.reg_play_control.enable = false;
                }
            }
            _ => logln!("VSU write byte not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value)
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        logln!("WARNING: Attempted read halfword from VSU (addr: 0x{:08x})", addr);

        0
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        self.write_byte(addr, value as _);
    }

    pub fn cycles(&mut self, num_cycles: usize, audio_frame_sink: &mut AudioFrameSink) {
        for _ in 0..num_cycles {
            self.duration_clock_counter += 1;
            if self.duration_clock_counter >= DURATION_CLOCK_PERIOD {
                self.duration_clock_counter = 0;

                self.voice1.reg_play_control.duration_clock();
                self.voice2.reg_play_control.duration_clock();
                self.voice3.reg_play_control.duration_clock();
                self.voice4.reg_play_control.duration_clock();
                self.voice5.reg_play_control.duration_clock();
                self.voice6.reg_play_control.duration_clock();
            }

            self.envelope_clock_counter += 1;
            if self.envelope_clock_counter >= ENVELOPE_CLOCK_PERIOD {
                self.envelope_clock_counter = 0;

                self.voice1.envelope.envelope_clock();
                self.voice2.envelope.envelope_clock();
                self.voice3.envelope.envelope_clock();
                self.voice4.envelope.envelope_clock();
                self.voice5.envelope.envelope_clock();
                self.voice6.envelope.envelope_clock();
            }

            self.frequency_clock_counter += 1;
            if self.frequency_clock_counter >= FREQUENCY_CLOCK_PERIOD {
                self.frequency_clock_counter = 0;

                self.voice1.frequency_clock();
                self.voice2.frequency_clock();
                self.voice3.frequency_clock();
                self.voice4.frequency_clock();
                self.voice5.frequency_clock();
            }

            self.noise_clock_counter += 1;
            if self.noise_clock_counter >= NOISE_CLOCK_PERIOD {
                self.noise_clock_counter = 0;

                self.voice6.noise_clock();
            }

            self.sample_clock_counter += 1;
            if self.sample_clock_counter >= SAMPLE_CLOCK_PERIOD {
                self.sample_clock_counter = 0;

                self.sample_clock(audio_frame_sink);
            }
        }
    }

    fn sample_clock(&mut self, audio_frame_sink: &mut AudioFrameSink) {
        let mut acc_left = 0;
        let mut acc_right = 0;

        fn mix_sample<V: Voice>(acc_left: &mut usize, acc_right: &mut usize, voice: &V, voice_output: usize) {
            let (left, right) = if voice.reg_play_control().enable {
                let envelope_level = voice.envelope().level();

                let left_level = if voice.reg_volume().left == 0 || envelope_level == 0 {
                    0
                } else {
                    ((voice.reg_volume().left * envelope_level) >> 3) + 1
                };
                let right_level = if voice.reg_volume().right == 0 || envelope_level == 0 {
                    0
                } else {
                    ((voice.reg_volume().right * envelope_level) >> 3) + 1
                };

                let output_left = (voice_output * left_level) >> 1;
                let output_right = (voice_output * right_level) >> 1;

                (output_left, output_right)
            } else {
                (0, 0)
            };

            *acc_left += left;
            *acc_right += right;
        }

        mix_sample(&mut acc_left, &mut acc_right, &self.voice1, self.voice1.output(&self.wave_tables));
        mix_sample(&mut acc_left, &mut acc_right, &self.voice2, self.voice2.output(&self.wave_tables));
        mix_sample(&mut acc_left, &mut acc_right, &self.voice3, self.voice3.output(&self.wave_tables));
        mix_sample(&mut acc_left, &mut acc_right, &self.voice4, self.voice4.output(&self.wave_tables));
        mix_sample(&mut acc_left, &mut acc_right, &self.voice5, self.voice5.output(&self.wave_tables));
        mix_sample(&mut acc_left, &mut acc_right, &self.voice6, self.voice6.output());

        let output_left = ((acc_left & 0xfff8) << 2) as i16;
        let output_right = ((acc_right & 0xfff8) << 2) as i16;

        audio_frame_sink.append((output_left, output_right));
    }

    fn are_channels_active(&self) -> bool {
        self.voice1.reg_play_control.enable ||
        self.voice2.reg_play_control.enable ||
        self.voice3.reg_play_control.enable ||
        self.voice4.reg_play_control.enable ||
        self.voice5.reg_play_control.enable ||
        self.voice6.reg_play_control.enable
    }
}
