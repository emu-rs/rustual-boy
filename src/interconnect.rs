use rom::*;

pub struct Interconnect {
    rom: Rom,
}

impl Interconnect {
    pub fn new(rom: Rom) -> Interconnect {
        Interconnect { rom: rom }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0x07fffffe;
        if addr >= 0x07000000 {
            let rom_bytes = self.rom.bytes();
            let rom_size = self.rom.size();
            let rom_mask = (rom_size - 1) as u32;
            let addr = addr & rom_mask;
            let low_byte = rom_bytes[addr as usize];
            let high_byte = rom_bytes[(addr + 1) as usize];
            ((high_byte as u16) << 8) | (low_byte as u16)
        } else {
            panic!("Unrecognized addr: {:#08x}", addr)
        }
    }
}
