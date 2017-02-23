// 20mhz / (1s / 100us) = 2000
const LARGE_INTERVAL_PERIOD: usize = 2000;

// 20mhz / (1s / 20us) = 400
const SMALL_INTERVAL_PERIOD: usize = 400;

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
    reload_pending: bool,
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
            reload: 0xffff,
            reload_pending: false,
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
        if ((value >> 2) & 0x01) != 0 {
            if self.counter != 0 {
                self.zero_status = false;
            }
        }
        if !self.zero_interrupt_enable || !self.zero_status {
            self.zero_interrupt = false;
        }
        self.enable = (value & 0x01) != 0;
    }

    pub fn read_counter_reload_low_reg(&self) -> u8 {
        self.counter as _
    }

    pub fn write_counter_reload_low_reg(&mut self, value: u8) {
        self.reload = (self.reload & 0xff00) | (value as u16);
        self.reload_pending = true;
    }

    pub fn read_counter_reload_high_reg(&self) -> u8 {
        (self.counter >> 8) as _
    }

    pub fn write_counter_reload_high_reg(&mut self, value: u8) {
        self.reload = ((value as u16) << 8) | (self.reload & 0xff);
        self.reload_pending = true;
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

                    self.counter = if self.reload_pending || self.counter == 0 {
                        self.reload_pending = false;
                        self.reload
                    } else {
                        self.counter - 1
                    };

                    self.zero_status = self.counter == 0;

                    if self.zero_status && self.zero_interrupt_enable {
                        self.zero_interrupt = true;
                    }
                }
            }
        }

        self.zero_interrupt
    }
}
