pub struct ComPort {
    pub cdtr: u8,
    pub cdrr: u8,

    pub c_stat: bool,

    pub transfer_bit_index: u32,
}

impl ComPort {
    pub fn new() -> ComPort {
        ComPort {
            cdtr: 0,
            cdrr: 0,

            c_stat: false,

            transfer_bit_index: 0,
        }
    }

    pub fn read_ccr(&self) -> u8 {
        logln!(Log::Ic, "WARNING: Read from CCR not fully implemented");
        0b01101001 |
        if self.c_stat { 1 << 1 } else { 0 }
    }

    pub fn write_ccr(&mut self, value: u8) {
        logln!(Log::Ic, "WARNING: Write to CCR not yet implemented (value: 0x{:02x})", value);
        if (value & 0x04) != 0 && !self.c_stat {
            self.cdrr = 0;
            self.c_stat = true;
            self.transfer_bit_index = 7;
        }
    }

    pub fn read_ccsr(&self) -> u8 {
        logln!(Log::Ic, "WARNING: Read from CCSR not yet implemented");
        0
    }

    pub fn write_ccsr(&mut self, value: u8) {
        logln!(Log::Ic, "WARNING: Write to CCSR not yet implemented (value: 0x{:02x})", value);
    }

    pub fn read_cdtr(&self) -> u8 {
        self.cdtr
    }

    pub fn write_cdtr(&mut self, value: u8) {
        self.cdtr = value;
    }

    pub fn read_cdrr(&self) -> u8 {
        self.cdrr
    }

    // TODO: This covers the case where the VB is slave only, and doesn't properly emulate any possible timing errors that might occur.
    pub fn transfer_slave_clock_bit(&mut self, bit: u32) -> u32 {
        if !self.c_stat {
            return 0;
        }

        self.cdrr |= (bit << self.transfer_bit_index) as u8;
        let ret = ((self.cdtr >> self.transfer_bit_index) & 1) as u32;

        if self.transfer_bit_index == 0 {
            self.c_stat = false;

            // TODO: Emulate link port interrupt (if applicable)
        } else {
            self.transfer_bit_index -= 1;
        }

        ret
    }
}
