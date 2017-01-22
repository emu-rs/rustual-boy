// 20mhz / (1s / 100us) = 2000
const LARGE_INTERVAL_PERIOD: usize = 2000;

// 20mhz / (1s / 16.66us repeating) = ~333
//  Note that the docs claim this interval is 20us rather than 16.66us repeating, but
//  it could be just rounding, as is often the case. This particular value comes from
//  extensive testing/tweaking comparing recordings of the the vocal sample in the
//  intro of Galactic Pinball from both the real hw and the emu with slightly different
//  intervals in the range of ~16-20ms.
const SMALL_INTERVAL_PERIOD: usize = 333;

enum Interval {
    Large,
    Small,
}

pub struct Timer {
    interval: Interval,
    zero_interrupt_enable: bool,
    zero_status: bool,
    enable: bool,
    reload: u16,
    counter: u16,

    tick_counter: usize,
    zero_interrupt: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            interval: Interval::Large,
            zero_interrupt_enable: false,
            zero_status: false,
            enable: false,
            reload: 0,
            counter: 0,

            tick_counter: 0,
            zero_interrupt: false,
        }
    }

    pub fn read_control_reg(&self) -> u8 {
        (match self.interval {
            Interval::Large => 0,
            Interval::Small => 1,
        } << 4) |
        (if self.zero_interrupt_enable { 1 } else { 0 } << 3) |
        (if self.zero_status { 1 } else { 0 } << 1) |
        if self.enable { 1 } else { 0 }
    }

    pub fn write_control_reg(&mut self, value: u8) {
        self.interval = if ((value >> 4) & 0x01) == 0 {
            Interval::Large
        } else {
            Interval::Small
        };
        self.zero_interrupt_enable = ((value >> 3) & 0x01) != 0;
        if !self.zero_interrupt_enable {
            self.zero_interrupt = false;
            if ((value >> 2) & 0x01) != 0 {
                self.zero_status = false;
            }
        }
        self.enable = (value & 0x01) != 0;
    }

    pub fn read_counter_reload_low_reg(&self) -> u8 {
        self.counter as _
    }

    pub fn write_counter_reload_low_reg(&mut self, value: u8) {
        self.reload = (self.reload & 0xff00) | (value as u16);
        self.counter = self.reload;
    }

    pub fn read_counter_reload_high_reg(&self) -> u8 {
        (self.counter >> 8) as _
    }

    pub fn write_counter_reload_high_reg(&mut self, value: u8) {
        self.reload = ((value as u16) << 8) | (self.reload & 0xff);
        self.counter = self.reload;
    }

    pub fn cycles(&mut self, cycles: usize) -> bool {
        if self.enable {
            for _ in 0..cycles {
                let tick_period = match self.interval {
                    Interval::Large => LARGE_INTERVAL_PERIOD,
                    Interval::Small => SMALL_INTERVAL_PERIOD,
                };
                self.tick_counter += 1;
                if self.tick_counter >= tick_period {
                    self.tick_counter = 0;

                    self.counter = match self.counter {
                        0 => {
                            self.zero_status = true;
                            if self.zero_interrupt_enable {
                                self.zero_interrupt = true;
                            }
                            self.reload
                        }
                        _ => self.counter - 1
                    };
                }
            }
        }

        self.zero_interrupt
    }
}
