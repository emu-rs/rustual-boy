mod mem_map;

use sinks::*;

use self::mem_map::*;

// Docs claim the sample rate is 41.7khz, but my calculations indicate it should be 41666.66hz repeating
//  (see SAMPLE_CLOCK_PERIOD calculation below), so we take the nearest whole-number sample rate to that.
//  Note that the documentation rounds values in a lot of places, so that's probably what happened here.
pub const SAMPLE_RATE: u32 = 41667;

// 20mhz / 41.7khz
const SAMPLE_CLOCK_PERIOD: u32 = 480;

// 20mhz / 260.4hz
const DURATION_CLOCK_PERIOD: u32 = SAMPLE_CLOCK_PERIOD * 160;

// 20mhz / 65.1hz
const ENVELOPE_CLOCK_PERIOD: u32 = SAMPLE_CLOCK_PERIOD * 640;

// 20mhz / 5mhz
const FREQUENCY_CLOCK_PERIOD: u32 = 4;

// 20mhz / 1041.6hz
const SWEEP_MOD_SMALL_PERIOD: u32 = SAMPLE_CLOCK_PERIOD * 40;

// 20mhz / 130.2hz
const SWEEP_MOD_LARGE_PERIOD: u32 = SAMPLE_CLOCK_PERIOD * 320;

// 20mhz / 500khz = 40 clocks
const NOISE_CLOCK_PERIOD: u32 = 40;

const NUM_WAVEFORM_DATA_WORDS: u32 = 32;
const NUM_WAVEFORM_DATA_TABLES: u32 = 5;
const TOTAL_WAVEFORM_DATA_SIZE: u32 = NUM_WAVEFORM_DATA_WORDS * NUM_WAVEFORM_DATA_TABLES;

const NUM_MOD_DATA_WORDS: u32 = 32;

#[derive(Default)]
struct IntReg {
    output_enable: bool,
    interval_data: bool,
    interval_counter_setting_values: u32,

    interval_counter: u32,
}

impl IntReg {
    fn write(&mut self, value: u8) {
        self.output_enable = (value & 0x80) != 0;
        self.interval_data = (value & 0x20) != 0;
        self.interval_counter_setting_values = (value & 0x1f) as _;

        if self.interval_data {
            self.interval_counter = 0;
        }
    }

    fn duration_clock(&mut self) {
        if self.output_enable && self.interval_data {
            self.interval_counter += 1;
            if self.interval_counter > self.interval_counter_setting_values {
                self.output_enable = false;
            }
        }
    }
}

#[derive(Default)]
struct LrvReg {
    left: u32,
    right: u32,
}

impl LrvReg {
    fn write(&mut self, value: u8) {
        self.left = (value >> 4) as _;
        self.right = (value & 0x0f) as _;
    }
}

#[derive(Default)]
struct Envelope {
    reg_data_reload: u32,
    reg_data_direction: bool,
    reg_data_step_interval: u32,

    reg_control_repeat: bool,
    reg_control_enable: bool,

    level: u32,

    envelope_counter: u32,
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

    fn level(&self) -> u32 {
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

trait Sound {
    fn reg_int(&self) -> &IntReg;
    fn reg_lrv(&self) -> &LrvReg;
    fn envelope(&self) -> &Envelope;
}

#[derive(Default)]
struct StandardSound {
    reg_int: IntReg,

    reg_lrv: LrvReg,

    fql: u32,
    fqh: u32,

    envelope: Envelope,

    ram: u32,

    frequency_counter: u32,
    phase: u32,
}

impl StandardSound {
    fn write_int(&mut self, value: u8) {
        self.reg_int.write(value);

        if self.reg_int.output_enable {
            self.envelope.envelope_counter = 0;

            self.frequency_counter = 0;
            self.phase = 0;
        }
    }

    fn write_lrv(&mut self, value: u8) {
        self.reg_lrv.write(value);
    }

    fn write_fql(&mut self, value: u8) {
        self.fql = value as _;
    }

    fn write_fqh(&mut self, value: u8) {
        self.fqh = (value & 0x07) as _;
    }

    fn write_ev0(&mut self, value: u8) {
        self.envelope.write_data_reg(value);
    }

    fn write_ev1(&mut self, value: u8) {
        self.envelope.write_control_reg(value);
    }

    fn write_ram(&mut self, value: u8) {
        self.ram = (value & 0x07) as _;
    }

    fn frequency_clock(&mut self) {
        self.frequency_counter += 1;
        if self.frequency_counter >= 2048 - ((self.fqh << 8) | self.fql) {
            self.frequency_counter = 0;

            self.phase = (self.phase + 1) & (NUM_WAVEFORM_DATA_WORDS - 1);
        }
    }

    fn output(&self, waveform_data: &[u8]) -> u32 {
        if self.ram > 4 {
            return 0;
        }

        waveform_data[(self.ram * NUM_WAVEFORM_DATA_WORDS + self.phase) as usize] as _
    }
}

impl Sound for StandardSound {
    fn reg_int(&self) -> &IntReg {
        &self.reg_int
    }

    fn reg_lrv(&self) -> &LrvReg {
        &self.reg_lrv
    }

    fn envelope(&self) -> &Envelope {
        &self.envelope
    }
}

#[derive(Default)]
struct SweepModSound {
    reg_int: IntReg,

    reg_lrv: LrvReg,

    fql: u32,
    fqh: u32,
    frequency_low: u32,
    frequency_high: u32,
    next_frequency_low: u32,
    next_frequency_high: u32,

    envelope: Envelope,

    reg_sweep_mod_enable: bool,
    reg_mod_repeat: bool,
    reg_function: bool,

    reg_sweep_mod_base_interval: bool,
    reg_sweep_mod_interval: u32,
    reg_sweep_direction: bool,
    reg_sweep_shift_amount: u32,

    ram: u32,

    frequency_counter: u32,
    phase: u32,

    sweep_mod_counter: u32,
    mod_phase: u32,
}

impl SweepModSound {
    fn write_int(&mut self, value: u8) {
        self.reg_int.write(value);

        if self.reg_int.output_enable {
            self.envelope.envelope_counter = 0;

            self.frequency_counter = 0;
            self.phase = 0;
            self.sweep_mod_counter = 0;
            self.mod_phase = 0;
        }
    }

    fn write_lrv(&mut self, value: u8) {
        self.reg_lrv.write(value);
    }

    fn write_fql(&mut self, value: u8) {
        self.fql = value as _;
        self.next_frequency_low = self.fql;
    }

    fn write_fqh(&mut self, value: u8) {
        self.fqh = (value & 0x07) as _;
        self.next_frequency_high = self.fqh;
    }

    fn write_ev0(&mut self, value: u8) {
        self.envelope.write_data_reg(value);
    }

    fn write_ev1(&mut self, value: u8) {
        self.envelope.write_control_reg(value);
        self.reg_sweep_mod_enable = ((value >> 6) & 0x01) != 0;
        self.reg_mod_repeat = ((value >> 5) & 0x01) != 0;
        self.reg_function = ((value >> 4) & 0x01) != 0;
    }

    fn write_swp(&mut self, value: u8) {
        self.reg_sweep_mod_base_interval = ((value >> 7) & 0x01) != 0;
        self.reg_sweep_mod_interval = ((value >> 4) & 0x07) as _;
        self.reg_sweep_direction = ((value >> 3) & 0x01) != 0;
        self.reg_sweep_shift_amount = (value & 0x07) as _;
    }

    fn write_ram(&mut self, value: u8) {
        self.ram = (value & 0x07) as _;
    }

    fn frequency_clock(&mut self) {
        self.frequency_counter += 1;
        if self.frequency_counter >= 2048 - ((self.frequency_high << 8) | self.frequency_low) {
            self.frequency_counter = 0;

            self.phase = (self.phase + 1) & (NUM_WAVEFORM_DATA_WORDS - 1);
        }
    }

    fn sweep_mod_clock(&mut self, mod_data: &[i8]) {
        self.sweep_mod_counter += 1;
        if self.sweep_mod_counter >= self.reg_sweep_mod_interval {
            self.sweep_mod_counter = 0;

            self.frequency_low = self.next_frequency_low;
            self.frequency_high = self.next_frequency_high;

            let mut freq = (self.frequency_high << 8) | self.frequency_low;

            if freq >= 2048 {
                self.reg_int.output_enable = false;
            }

            if !self.reg_int.output_enable || !self.reg_sweep_mod_enable || self.reg_sweep_mod_interval == 0 {
                return;
            }

            match self.reg_function {
                false => {
                    // Sweep
                    let sweep_value = freq >> self.reg_sweep_shift_amount;
                    freq = match self.reg_sweep_direction {
                        false => freq.wrapping_sub(sweep_value),
                        true => freq.wrapping_add(sweep_value)
                    };
                }
                true => {
                    // Mod
                    let reg_freq = (self.fqh << 8) | self.fql;
                    freq = reg_freq.wrapping_add(mod_data[self.mod_phase as usize] as _) & 0x07ff;

                    const MAX_MOD_PHASE: u32 = NUM_MOD_DATA_WORDS - 1;
                    self.mod_phase = match (self.reg_mod_repeat, self.mod_phase) {
                        (false, MAX_MOD_PHASE) => MAX_MOD_PHASE,
                        _ => (self.mod_phase + 1) & MAX_MOD_PHASE
                    };
                }
            }

            self.next_frequency_low = freq & 0xff;
            self.next_frequency_high = (freq >> 8) & 0x07;
        }
    }

    fn output(&self, waveform_data: &[u8]) -> u32 {
        if self.ram > 4 {
            return 0;
        }

        waveform_data[(self.ram * NUM_WAVEFORM_DATA_WORDS + self.phase) as usize] as _
    }
}

impl Sound for SweepModSound {
    fn reg_int(&self) -> &IntReg {
        &self.reg_int
    }

    fn reg_lrv(&self) -> &LrvReg {
        &self.reg_lrv
    }

    fn envelope(&self) -> &Envelope {
        &self.envelope
    }
}

#[derive(Default)]
struct NoiseSound {
    reg_int: IntReg,

    reg_lrv: LrvReg,

    fql: u32,
    fqh: u32,

    envelope: Envelope,

    reg_noise_control: u32,

    frequency_counter: u32,
    shift: u32,
    output: u32,
}

impl NoiseSound {
    fn write_int(&mut self, value: u8) {
        self.reg_int.write(value);

        if self.reg_int.output_enable {
            self.envelope.envelope_counter = 0;

            self.frequency_counter = 0;
            self.shift = 0x7fff;
        }
    }

    fn write_lrv(&mut self, value: u8) {
        self.reg_lrv.write(value);
    }

    fn write_fql(&mut self, value: u8) {
        self.fql = value as _;
    }

    fn write_fqh(&mut self, value: u8) {
        self.fqh = (value & 0x07) as _;
    }

    fn write_ev0(&mut self, value: u8) {
        self.envelope.write_data_reg(value);
    }

    fn write_ev1(&mut self, value: u8) {
        self.reg_noise_control = ((value >> 4) & 0x07) as _;
        self.envelope.write_control_reg(value);
    }

    fn noise_clock(&mut self) {
        self.frequency_counter += 1;
        if self.frequency_counter >= 2048 - ((self.fqh << 8) | self.fql) {
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

    fn output(&self) -> u32 {
        self.output
    }
}

impl Sound for NoiseSound {
    fn reg_int(&self) -> &IntReg {
        &self.reg_int
    }

    fn reg_lrv(&self) -> &LrvReg {
        &self.reg_lrv
    }

    fn envelope(&self) -> &Envelope {
        &self.envelope
    }
}

pub struct Vsu {
    waveform_data: Box<[u8]>,
    mod_data: Box<[i8]>,

    sound1: StandardSound,
    sound2: StandardSound,
    sound3: StandardSound,
    sound4: StandardSound,
    sound5: SweepModSound,
    sound6: NoiseSound,

    duration_clock_counter: u32,
    envelope_clock_counter: u32,
    frequency_clock_counter: u32,
    sweep_mod_clock_counter: u32,
    noise_clock_counter: u32,
    sample_clock_counter: u32,
}

impl Vsu {
    pub fn new() -> Vsu {
        Vsu {
            waveform_data: vec![0; TOTAL_WAVEFORM_DATA_SIZE as usize].into_boxed_slice(),
            mod_data: vec![0; NUM_MOD_DATA_WORDS as usize].into_boxed_slice(),

            sound1: StandardSound::default(),
            sound2: StandardSound::default(),
            sound3: StandardSound::default(),
            sound4: StandardSound::default(),
            sound5: SweepModSound::default(),
            sound6: NoiseSound::default(),

            duration_clock_counter: 0,
            envelope_clock_counter: 0,
            frequency_clock_counter: 0,
            sweep_mod_clock_counter: 0,
            noise_clock_counter: 0,
            sample_clock_counter: 0,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        logln!(Log::Vsu, "WARNING: Attempted read byte from VSU (addr: 0x{:08x})", addr);

        0
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match addr {
            WAVEFORM_DATA_0_START ..= WAVEFORM_DATA_0_END => {
                if !self.are_channels_active() {
                    self.waveform_data[((addr - WAVEFORM_DATA_0_START) / 4 + 0x00) as usize] = value & 0x3f;
                }
            }
            WAVEFORM_DATA_1_START ..= WAVEFORM_DATA_1_END => {
                if !self.are_channels_active() {
                    self.waveform_data[((addr - WAVEFORM_DATA_1_START) / 4 + 0x20) as usize] = value & 0x3f;
                }
            }
            WAVEFORM_DATA_2_START ..= WAVEFORM_DATA_2_END => {
                if !self.are_channels_active() {
                    self.waveform_data[((addr - WAVEFORM_DATA_2_START) / 4 + 0x40) as usize] = value & 0x3f;
                }
            }
            WAVEFORM_DATA_3_START ..= WAVEFORM_DATA_3_END => {
                if !self.are_channels_active() {
                    self.waveform_data[((addr - WAVEFORM_DATA_3_START) / 4 + 0x60) as usize] = value & 0x3f;
                }
            }
            WAVEFORM_DATA_4_START ..= WAVEFORM_DATA_4_END => {
                if !self.are_channels_active() {
                    self.waveform_data[((addr - WAVEFORM_DATA_4_START) / 4 + 0x80) as usize] = value & 0x3f;
                }
            }
            MOD_DATA_START ..= MOD_DATA_END => {
                if !self.are_channels_active() {
                    self.mod_data[((addr - MOD_DATA_START) / 4) as usize] = value as _;
                }
            }
            S1INT => self.sound1.write_int(value),
            S1LRV => self.sound1.write_lrv(value),
            S1FQL => self.sound1.write_fql(value),
            S1FQH => self.sound1.write_fqh(value),
            S1EV0 => self.sound1.write_ev0(value),
            S1EV1 => self.sound1.write_ev1(value),
            S1RAM => self.sound1.write_ram(value),
            S2INT => self.sound2.write_int(value),
            S2LRV => self.sound2.write_lrv(value),
            S2FQL => self.sound2.write_fql(value),
            S2FQH => self.sound2.write_fqh(value),
            S2EV0 => self.sound2.write_ev0(value),
            S2EV1 => self.sound2.write_ev1(value),
            S2RAM => self.sound2.write_ram(value),
            S3INT => self.sound3.write_int(value),
            S3LRV => self.sound3.write_lrv(value),
            S3FQL => self.sound3.write_fql(value),
            S3FQH => self.sound3.write_fqh(value),
            S3EV0 => self.sound3.write_ev0(value),
            S3EV1 => self.sound3.write_ev1(value),
            S3RAM => self.sound3.write_ram(value),
            S4INT => self.sound4.write_int(value),
            S4LRV => self.sound4.write_lrv(value),
            S4FQL => self.sound4.write_fql(value),
            S4FQH => self.sound4.write_fqh(value),
            S4EV0 => self.sound4.write_ev0(value),
            S4EV1 => self.sound4.write_ev1(value),
            S4RAM => self.sound4.write_ram(value),
            S5INT => self.sound5.write_int(value),
            S5LRV => self.sound5.write_lrv(value),
            S5FQL => self.sound5.write_fql(value),
            S5FQH => self.sound5.write_fqh(value),
            S5EV0 => self.sound5.write_ev0(value),
            S5EV1 => self.sound5.write_ev1(value),
            S5SWP => self.sound5.write_swp(value),
            S5RAM => self.sound5.write_ram(value),
            S6INT => self.sound6.write_int(value),
            S6LRV => self.sound6.write_lrv(value),
            S6FQL => self.sound6.write_fql(value),
            S6FQH => self.sound6.write_fqh(value),
            S6EV0 => self.sound6.write_ev0(value),
            S6EV1 => self.sound6.write_ev1(value),
            SSTOP => {
                if (value & 0x01) != 0 {
                    self.sound1.reg_int.output_enable = false;
                    self.sound2.reg_int.output_enable = false;
                    self.sound3.reg_int.output_enable = false;
                    self.sound4.reg_int.output_enable = false;
                    self.sound5.reg_int.output_enable = false;
                    self.sound6.reg_int.output_enable = false;
                }
            }
            _ => logln!(Log::Vsu, "VSU write byte not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value)
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        logln!(Log::Vsu, "WARNING: Attempted read halfword from VSU (addr: 0x{:08x})", addr);

        0
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        self.write_byte(addr, value as _);
    }

    pub fn cycles(&mut self, num_cycles: u32, audio_frame_sink: &mut dyn Sink<AudioFrame>) {
        for _ in 0..num_cycles {
            self.duration_clock_counter += 1;
            if self.duration_clock_counter >= DURATION_CLOCK_PERIOD {
                self.duration_clock_counter = 0;

                self.sound1.reg_int.duration_clock();
                self.sound2.reg_int.duration_clock();
                self.sound3.reg_int.duration_clock();
                self.sound4.reg_int.duration_clock();
                self.sound5.reg_int.duration_clock();
                self.sound6.reg_int.duration_clock();
            }

            self.envelope_clock_counter += 1;
            if self.envelope_clock_counter >= ENVELOPE_CLOCK_PERIOD {
                self.envelope_clock_counter = 0;

                self.sound1.envelope.envelope_clock();
                self.sound2.envelope.envelope_clock();
                self.sound3.envelope.envelope_clock();
                self.sound4.envelope.envelope_clock();
                self.sound5.envelope.envelope_clock();
                self.sound6.envelope.envelope_clock();
            }

            self.frequency_clock_counter += 1;
            if self.frequency_clock_counter >= FREQUENCY_CLOCK_PERIOD {
                self.frequency_clock_counter = 0;

                self.sound1.frequency_clock();
                self.sound2.frequency_clock();
                self.sound3.frequency_clock();
                self.sound4.frequency_clock();
                self.sound5.frequency_clock();
            }

            self.sweep_mod_clock_counter += 1;
            let sweep_mod_clock_period = match self.sound5.reg_sweep_mod_base_interval {
                false => SWEEP_MOD_SMALL_PERIOD,
                true => SWEEP_MOD_LARGE_PERIOD
            };
            if self.sweep_mod_clock_counter >= sweep_mod_clock_period {
                self.sweep_mod_clock_counter = 0;

                self.sound5.sweep_mod_clock(&self.mod_data);
            }

            self.noise_clock_counter += 1;
            if self.noise_clock_counter >= NOISE_CLOCK_PERIOD {
                self.noise_clock_counter = 0;

                self.sound6.noise_clock();
            }

            self.sample_clock_counter += 1;
            if self.sample_clock_counter >= SAMPLE_CLOCK_PERIOD {
                self.sample_clock_counter = 0;

                self.sample_clock(audio_frame_sink);
            }
        }
    }

    fn sample_clock(&mut self, audio_frame_sink: &mut dyn Sink<AudioFrame>) {
        let mut acc_left = 0;
        let mut acc_right = 0;

        fn mix_sample<S: Sound>(acc_left: &mut u32, acc_right: &mut u32, sound: &S, sound_output: u32) {
            let (left, right) = if sound.reg_int().output_enable {
                let envelope_level = sound.envelope().level();

                let left_level = if sound.reg_lrv().left == 0 || envelope_level == 0 {
                    0
                } else {
                    ((sound.reg_lrv().left * envelope_level) >> 3) + 1
                };
                let right_level = if sound.reg_lrv().right == 0 || envelope_level == 0 {
                    0
                } else {
                    ((sound.reg_lrv().right * envelope_level) >> 3) + 1
                };

                let output_left = (sound_output * left_level) >> 1;
                let output_right = (sound_output * right_level) >> 1;

                (output_left, output_right)
            } else {
                (0, 0)
            };

            *acc_left += left;
            *acc_right += right;
        }

        mix_sample(&mut acc_left, &mut acc_right, &self.sound1, self.sound1.output(&self.waveform_data));
        mix_sample(&mut acc_left, &mut acc_right, &self.sound2, self.sound2.output(&self.waveform_data));
        mix_sample(&mut acc_left, &mut acc_right, &self.sound3, self.sound3.output(&self.waveform_data));
        mix_sample(&mut acc_left, &mut acc_right, &self.sound4, self.sound4.output(&self.waveform_data));
        mix_sample(&mut acc_left, &mut acc_right, &self.sound5, self.sound5.output(&self.waveform_data));
        mix_sample(&mut acc_left, &mut acc_right, &self.sound6, self.sound6.output());

        let output_left = ((acc_left & 0xfff8) << 2) as i16;
        let output_right = ((acc_right & 0xfff8) << 2) as i16;

        audio_frame_sink.append((output_left, output_right));
    }

    fn are_channels_active(&self) -> bool {
        self.sound1.reg_int.output_enable ||
        self.sound2.reg_int.output_enable ||
        self.sound3.reg_int.output_enable ||
        self.sound4.reg_int.output_enable ||
        self.sound5.reg_int.output_enable ||
        self.sound6.reg_int.output_enable
    }
}
