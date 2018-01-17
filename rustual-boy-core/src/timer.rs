// 20mhz / (1s / 100us) = 2000
const LARGE_INTERVAL_PERIOD: u32 = 2000;

// 20mhz / (1s / 20us) = 400
const SMALL_INTERVAL_PERIOD: u32 = 400;

enum Interval {
    Large,
    Small,
}

pub struct Timer {
    t_clk_sel: Interval,
    tim_z_int: bool,
    z_stat: bool,
    t_enb: bool,
    reload: u16,
    counter: u16,

    tick_counter: u32,
    zero_interrupt: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            t_clk_sel: Interval::Large,
            tim_z_int: false,
            z_stat: false,
            t_enb: false,
            reload: 0,
            counter: 0xffff,

            tick_counter: 0,
            zero_interrupt: false,
        }
    }

    pub fn read_tcr(&self) -> u8 {
        0b11100100 |
        (match self.t_clk_sel {
            Interval::Large => 0,
            Interval::Small => 1,
        } << 4) |
        (if self.tim_z_int { 1 } else { 0 } << 3) |
        (if self.z_stat { 1 } else { 0 } << 1) |
        if self.t_enb { 1 } else { 0 }
    }

    pub fn write_tcr(&mut self, value: u8) {
        self.t_clk_sel = if ((value >> 4) & 0x01) == 0 {
            Interval::Large
        } else {
            Interval::Small
        };
        self.tim_z_int = ((value >> 3) & 0x01) != 0;
        if ((value >> 2) & 0x01) != 0 {
            self.z_stat = false;
        }
        if !self.tim_z_int || !self.z_stat {
            self.zero_interrupt = false;
        }
        self.t_enb = (value & 0x01) != 0;
    }

    pub fn read_tlr(&self) -> u8 {
        self.counter as _
    }

    pub fn write_tlr(&mut self, value: u8) {
        self.reload = (self.reload & 0xff00) | (value as u16);
        self.counter = self.reload;
    }

    pub fn read_thr(&self) -> u8 {
        (self.counter >> 8) as _
    }

    pub fn write_thr(&mut self, value: u8) {
        self.reload = ((value as u16) << 8) | (self.reload & 0xff);
        self.counter = self.reload;
    }

    pub fn cycles(&mut self, cycles: u32) -> bool {
        if self.t_enb {
            for _ in 0..cycles {
                let tick_period = match self.t_clk_sel {
                    Interval::Large => LARGE_INTERVAL_PERIOD,
                    Interval::Small => SMALL_INTERVAL_PERIOD,
                };
                self.tick_counter += 1;
                if self.tick_counter >= tick_period {
                    self.tick_counter = 0;

                    self.counter = match self.counter {
                        0 => {
                            self.z_stat = true;
                            if self.tim_z_int {
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
