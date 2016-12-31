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
        println!("VSU read byte not yet implemented (addr: 0x{:08x})", addr);
        0
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        println!("VSU write byte not yet implemented (addr: 0x{:08x}, value: 0x{:02x})", addr, value);
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        println!("VSU read halfword not yet implemented (addr: 0x{:08x})", addr);
        0
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        println!("VSU write halfword not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
    }

    pub fn read_word(&self, addr: u32) -> u32 {
        let addr = addr & 0xfffffffc;
        println!("VSU read word not yet implemented (addr: 0x{:08x})", addr);
        0
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        let addr = addr & 0xfffffffc;
        println!("VSU write word not yet implemented (addr: 0x{:08x}, value: 0x{:08x})", addr, value);
    }
}