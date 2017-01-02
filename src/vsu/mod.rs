pub struct Vsu {
    // TODO
}

impl Vsu {
    pub fn new() -> Vsu {
        Vsu {
            // TODO
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        logln!("VSU read byte not yet implemented (addr: 0x{:08x})", addr);
        0
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        logln!("VSU write byte not yet implemented (addr: 0x{:08x}, value: 0x{:02x})", addr, value);
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        logln!("VSU read halfword not yet implemented (addr: 0x{:08x})", addr);
        0
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        logln!("VSU write halfword not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
    }
}