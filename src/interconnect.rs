use rom::*;

pub struct Interconnect {
    rom: Rom,
}

impl Interconnect {
    pub fn new(rom: Rom) -> Interconnect {
        Interconnect { rom: rom }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let addr = addr & 0x07ffffff;
        if addr >= 0x07000000 {
            let rom_bytes = self.rom.bytes();
            let rom_size = self.rom.size();
            let rom_mask = (rom_size - 1) as u32;
            let addr = addr & rom_mask;
            rom_bytes[addr as usize]
        } else {
            panic!("Unrecognized addr: {:#08x}", addr);
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        let low_byte = self.read_byte(addr);
        let high_byte = self.read_byte(addr + 1);
        ((high_byte as u16) << 8) | (low_byte as u16)
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = addr & 0x07ffffff;
        panic!("Unrecognized addr: {:#08x}", addr);
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        let addr = addr & 0xfffffffc;
        self.write_byte(addr, value as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
        self.write_byte(addr + 2, (value >> 16) as u8);
        self.write_byte(addr + 3, (value >> 24) as u8);
    }

    pub fn cycles(&mut self, cycles: usize) {
        // TODO
    }
}
