use rom::*;
use wram::*;
use mem_map::*;

pub struct Interconnect {
    rom: Rom,
    wram: Wram,
}

impl Interconnect {
    pub fn new(rom: Rom) -> Interconnect {
        Interconnect {
            rom: rom,
            wram: Wram::new(),
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        match map_address(addr) {
            MappedAddress::WaitControlReg => {
                panic!("Read byte from Wait Control Register not yet implemented");
            }
            MappedAddress::Wram(addr) => self.wram.read_byte(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_byte(addr),
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::WaitControlReg => {
                panic!("Read halfword from Wait Control Register not yet implemented");
            }
            MappedAddress::Wram(addr) => self.wram.read_halfword(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_halfword(addr),
        }
    }

    pub fn read_word(&self, addr: u32) -> u32 {
        let addr = addr & 0xfffffffc;
        match map_address(addr) {
            MappedAddress::WaitControlReg => {
                panic!("Read word from Wait Control Register not yet implemented");
            }
            MappedAddress::Wram(addr) => self.wram.read_word(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_word(addr),
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match map_address(addr) {
            MappedAddress::WaitControlReg => {
                println!("Wait Control Register (0x{:08x}) written: 0x{:02x}", addr, value);
                println!(" Cartridge ROM Waits: {}", if value & 0x01 == 0 { 2 } else { 1 });
                println!(" Cartridge Expansion Waits: {}", if value & 0x02 == 0 { 2 } else { 1 });
            }
            MappedAddress::Wram(addr) => self.wram.write_byte(addr, value),
            MappedAddress::CartridgeRom(_) => {
                println!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr);
            }
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::WaitControlReg => {
                panic!("Write halfword to Wait Control Register not yet implemented");
            }
            MappedAddress::Wram(addr) => self.wram.write_halfword(addr, value),
            MappedAddress::CartridgeRom(_) => {
                println!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr);
            }
        }
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::WaitControlReg => {
                panic!("Write word to Wait Control Register not yet implemented");
            }
            MappedAddress::Wram(addr) => self.wram.write_word(addr, value),
            MappedAddress::CartridgeRom(_) => {
                println!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr);
            }
        }
    }

    pub fn cycles(&mut self, _cycles: usize) {
        // TODO
    }
}
