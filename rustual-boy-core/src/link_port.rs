pub struct LinkPort {
    transmit_data_reg: u8,
    receive_data_reg: u8,

    is_transfer_active: bool,
    transfer_bit_index: u32,
}

impl LinkPort {
    pub fn new() -> LinkPort {
        LinkPort {
            transmit_data_reg: 0,
            receive_data_reg: 0,

            is_transfer_active: false,
            transfer_bit_index: 0,
        }
    }

    pub fn read_control_reg(&self) -> u8 {
        logln!(Log::Ic, "WARNING: Read from Link Control Register not fully implemented");
        if self.is_transfer_active { 1 << 1 } else { 0 }
    }

    pub fn write_control_reg(&mut self, value: u8) {
        logln!(Log::Ic, "WARNING: Write to Link Control Register not yet implemented (value: 0x{:02x})", value);
        if (value & 0x04) != 0 && !self.is_transfer_active {
            self.receive_data_reg = 0;
            self.is_transfer_active = true;
            self.transfer_bit_index = 7;
        }
    }

    pub fn read_aux_reg(&self) -> u8 {
        logln!(Log::Ic, "WARNING: Read from Auxiliary Link Register not yet implemented");
        0
    }

    pub fn write_aux_reg(&mut self, value: u8) {
        logln!(Log::Ic, "WARNING: Write to Auxiliary Link Register not yet implemented (value: 0x{:02x})", value);
    }

    pub fn read_transmit_data_reg(&self) -> u8 {
        self.transmit_data_reg
    }

    pub fn write_transmit_data_reg(&mut self, value: u8) {
        self.transmit_data_reg = value;
    }

    pub fn read_receive_data_reg(&self) -> u8 {
        self.receive_data_reg
    }

    // TODO: This covers the case where the VB is slave only, and doesn't properly emulate any possible timing errors that might occur.
    pub fn transfer_slave_clock_bit(&mut self, bit: u32) -> u32 {
        if !self.is_transfer_active {
            return 0;
        }

        self.receive_data_reg |= (bit << self.transfer_bit_index) as u8;
        let ret = ((self.transmit_data_reg >> self.transfer_bit_index) & 1) as u32;

        if self.transfer_bit_index == 0 {
            self.is_transfer_active = false;

            // TODO: Emulate link port interrupt (if applicable)
        } else {
            self.transfer_bit_index -= 1;
        }

        ret
    }
}
