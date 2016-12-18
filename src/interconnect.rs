use rom::*;
use wram::*;
use vip::*;
use vsu::*;
use mem_map::*;

pub struct Interconnect {
    rom: Rom,
    wram: Wram,
    vip: Vip,
    vsu: Vsu,
}

impl Interconnect {
    pub fn new(rom: Rom) -> Interconnect {
        Interconnect {
            rom: rom,
            wram: Wram::new(),
            vip: Vip::new(),
            vsu: Vsu::new(),
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        match map_address(addr) {
            MappedAddress::Vip(addr) => self.vip.read_byte(addr),
            MappedAddress::Vsu(addr) => self.vsu.read_byte(addr),
            MappedAddress::LinkControlReg => {
                println!("WARNING: Read byte from Link Control Register not yet implemented");
                0
            }
            MappedAddress::AuxLinkReg => {
                println!("WARNING: Read byte from Auxiliary Link Register not yet implemented");
                0
            }
            MappedAddress::LinkTransmitDataReg => {
                panic!("Read byte from Link Transmit Data Register not yet implemented");
            }
            MappedAddress::LinkReceiveDataReg => {
                panic!("Read byte from Link Receive Data Register not yet implemented");
            }
            MappedAddress::GamePadInputLowReg => {
                println!("WARNING: Read byte from Game Pad Input Low Register not yet implemented");
                0
            }
            MappedAddress::GamePadInputHighReg => {
                println!("WARNING: Read byte from Game Pad Input High Register not yet implemented");
                0
            }
            MappedAddress::TimerCounterReloadLowReg => {
                println!("WARNING: Read byte from Timer Counter/Reload Low Register not yet implemented");
                0
            }
            MappedAddress::TimerCounterReloadHighReg => {
                println!("WARNING: Read byte from Timer Counter/Reload High Register not yet implemented");
                0
            }
            MappedAddress::TimerControlReg => {
                println!("WARNING: Read byte from Timer Control Register not yet implemented");
                0
            }
            MappedAddress::WaitControlReg => {
                panic!("Read byte from Wait Control Register not yet implemented");
            }
            MappedAddress::GamePadInputControlReg => {
                println!("WARNING: Read byte from Game Pad Input Control Register not yet implemented");
                0
            }
            MappedAddress::Wram(addr) => self.wram.read_byte(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_byte(addr),
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::Vip(addr) => self.vip.read_halfword(addr),
            MappedAddress::Vsu(addr) => self.vsu.read_halfword(addr),
            MappedAddress::LinkControlReg => {
                panic!("Read halfword from Link Control Register not yet implemented");
            }
            MappedAddress::AuxLinkReg => {
                panic!("Read halfword from Auxiliary Link Register not yet implemented");
            }
            MappedAddress::LinkTransmitDataReg => {
                panic!("Read halfword from Link Transmit Data Register not yet implemented");
            }
            MappedAddress::LinkReceiveDataReg => {
                panic!("Read halfword from Link Receive Data Register not yet implemented");
            }
            MappedAddress::GamePadInputLowReg => {
                panic!("Read halfword from Game Pad Input Low Register not yet implemented");
            }
            MappedAddress::GamePadInputHighReg => {
                panic!("Read halfword from Game Pad Input High Register not yet implemented");
            }
            MappedAddress::TimerCounterReloadLowReg => {
                println!("Read halfword from Timer Counter/Reload Low Register not yet implemented");
                0
            }
            MappedAddress::TimerCounterReloadHighReg => {
                panic!("Read halfword from Timer Counter/Reload High Register not yet implemented");
            }
            MappedAddress::TimerControlReg => {
                panic!("Read halfword from Timer Control Register not yet implemented");
            }
            MappedAddress::WaitControlReg => {
                panic!("Read halfword from Wait Control Register not yet implemented");
            }
            MappedAddress::GamePadInputControlReg => {
                panic!("Read halfword from Game Pad Input Control Register not yet implemented");
            }
            MappedAddress::Wram(addr) => self.wram.read_halfword(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_halfword(addr),
        }
    }

    pub fn read_word(&self, addr: u32) -> u32 {
        let addr = addr & 0xfffffffc;
        match map_address(addr) {
            MappedAddress::Vip(addr) => self.vip.read_word(addr),
            MappedAddress::Vsu(addr) => self.vsu.read_word(addr),
            MappedAddress::LinkControlReg => {
                panic!("Read word from Link Control Register not yet implemented");
            }
            MappedAddress::AuxLinkReg => {
                panic!("Read word from Auxiliary Link Register not yet implemented");
            }
            MappedAddress::LinkTransmitDataReg => {
                panic!("Read word from Link Transmit Data Register not yet implemented");
            }
            MappedAddress::LinkReceiveDataReg => {
                panic!("Read word from Link Receive Data Register not yet implemented");
            }
            MappedAddress::GamePadInputLowReg => {
                panic!("Read word from Game Pad Input Low Register not yet implemented");
            }
            MappedAddress::GamePadInputHighReg => {
                panic!("Read word from Game Pad Input High Register not yet implemented");
            }
            MappedAddress::TimerCounterReloadLowReg => {
                panic!("Read word from Timer Counter/Reload Low Register not yet implemented");
            }
            MappedAddress::TimerCounterReloadHighReg => {
                panic!("Read word from Timer Counter/Reload High Register not yet implemented");
            }
            MappedAddress::TimerControlReg => {
                panic!("Read word from Timer Control Register not yet implemented");
            }
            MappedAddress::WaitControlReg => {
                panic!("Read word from Wait Control Register not yet implemented");
            }
            MappedAddress::GamePadInputControlReg => {
                panic!("Read word from Game Pad Input Control Register not yet implemented");
            }
            MappedAddress::Wram(addr) => self.wram.read_word(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_word(addr),
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match map_address(addr) {
            MappedAddress::Vip(addr) => self.vip.write_byte(addr, value),
            MappedAddress::Vsu(addr) => self.vsu.write_byte(addr, value),
            MappedAddress::LinkControlReg => {
                println!("WARNING: Write byte to Link Control Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::AuxLinkReg => {
                println!("WARNING: Write byte to Auxiliary Link Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::LinkTransmitDataReg => {
                println!("WARNING: Write byte to Link Transmit Data Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::LinkReceiveDataReg => {
                println!("WARNING: Write byte to Link Receive Data Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::GamePadInputLowReg => {
                println!("WARNING: Write byte to Game Pad Input Low Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::GamePadInputHighReg => {
                println!("WARNING: Write byte to Game Pad Input High Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::TimerCounterReloadLowReg => {
                println!("WARNING: Write byte to Timer Counter/Reload Low Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::TimerCounterReloadHighReg => {
                println!("WARNING: Write byte to Timer Counter/Reload High Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::TimerControlReg => {
                println!("WARNING: Write byte to Timer Control Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::WaitControlReg => {
                println!("Wait Control Register (0x{:08x}) written: 0x{:02x}", addr, value);
                println!(" Cartridge ROM Waits: {}", if value & 0x01 == 0 { 2 } else { 1 });
                println!(" Cartridge Expansion Waits: {}", if value & 0x02 == 0 { 2 } else { 1 });
            }
            MappedAddress::GamePadInputControlReg => {
                println!("WARNING: Write byte to Game Pad Input Control Register not yet implemented (value: 0x{:02x})", value);
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
            MappedAddress::Vip(addr) => self.vip.write_halfword(addr, value),
            MappedAddress::Vsu(addr) => self.vsu.write_halfword(addr, value),
            MappedAddress::LinkControlReg => {
                println!("WARNING: Write halfword to Link Control Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::AuxLinkReg => {
                println!("WARNING: Write halfword to Auxiliary Link Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::LinkTransmitDataReg => {
                println!("WARNING: Write halfword to Link Transmit Data Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::LinkReceiveDataReg => {
                println!("WARNING: Write halfword to Link Receive Data Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::GamePadInputLowReg => {
                println!("WARNING: Write halfword to Game Pad Input Low Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::GamePadInputHighReg => {
                println!("WARNING: Write halfword to Game Pad Input High Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::TimerCounterReloadLowReg => {
                println!("WARNING: Write halfword to Timer Counter/Reload Low Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::TimerCounterReloadHighReg => {
                println!("WARNING: Write halfword to Timer Counter/Reload High Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::TimerControlReg => {
                println!("WARNING: Write halfword to Timer Control Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::WaitControlReg => {
                println!("WARNING: Write halfword to Wait Control Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::GamePadInputControlReg => {
                println!("WARNING: Write halfword to Game Pad Input Control Register not yet implemented (value: 0x{:04x})", value);
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
            MappedAddress::Vip(addr) => self.vip.write_word(addr, value),
            MappedAddress::Vsu(addr) => self.vsu.write_word(addr, value),
            MappedAddress::LinkControlReg => {
                println!("WARNING: Write word to Link Control Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::AuxLinkReg => {
                println!("WARNING: Write word to Auxiliary Link Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::LinkTransmitDataReg => {
                println!("WARNING: Write word to Link Transmit Data Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::LinkReceiveDataReg => {
                println!("WARNING: Write word to Link Receive Data Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::GamePadInputLowReg => {
                println!("WARNING: Write word to Game Pad Input Low Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::GamePadInputHighReg => {
                println!("WARNING: Write word to Game Pad Input High Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::TimerCounterReloadLowReg => {
                println!("WARNING: Write word to Timer Counter/Reload Low Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::TimerCounterReloadHighReg => {
                println!("WARNING: Write word to Timer Counter/Reload High Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::TimerControlReg => {
                println!("WARNING: Write word to Timer Control Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::WaitControlReg => {
                panic!("Write word to Wait Control Register not yet implemented");
            }
            MappedAddress::GamePadInputControlReg => {
                println!("WARNING: Write word to Game Pad Input Control Register not yet implemented (value: 0x{:08x})", value);
            }
            MappedAddress::Wram(addr) => self.wram.write_word(addr, value),
            MappedAddress::CartridgeRom(_) => {
                println!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr);
            }
        }
    }

    pub fn cycles(&mut self, cycles: usize) -> Option<u16> {
        if self.vip.cycles(cycles) {
            Some(0xfe40)
        } else {
            None
        }
    }
}
