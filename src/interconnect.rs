use video_driver::*;
use rom::*;
use wram::*;
use sram::*;
use vip::*;
use vsu::*;
use timer::*;
use game_pad::*;
use mem_map::*;

pub struct Interconnect {
    rom: Rom,
    wram: Wram,
    pub sram: Sram,
    vip: Vip,
    vsu: Vsu,
    timer: Timer,
    pub game_pad: GamePad,
}

impl Interconnect {
    pub fn new(rom: Rom, sram: Sram) -> Interconnect {
        Interconnect {
            rom: rom,
            wram: Wram::new(),
            sram: sram,
            vip: Vip::new(),
            vsu: Vsu::new(),
            timer: Timer::new(),
            game_pad: GamePad::new(),
        }
    }

    pub fn read_byte(&mut self, addr: u32) -> u8 {
        match map_address(addr) {
            MappedAddress::Vip(addr) => self.vip.read_byte(addr),
            MappedAddress::Vsu(addr) => self.vsu.read_byte(addr),
            MappedAddress::LinkControlReg => {
                logln!("WARNING: Read byte from Link Control Register not yet implemented");
                0
            }
            MappedAddress::AuxLinkReg => {
                logln!("WARNING: Read byte from Auxiliary Link Register not yet implemented");
                0
            }
            MappedAddress::LinkTransmitDataReg => {
                panic!("Read byte from Link Transmit Data Register not yet implemented");
            }
            MappedAddress::LinkReceiveDataReg => {
                panic!("Read byte from Link Receive Data Register not yet implemented");
            }
            MappedAddress::GamePadInputLowReg => self.game_pad.read_input_low_reg(),
            MappedAddress::GamePadInputHighReg => self.game_pad.read_input_high_reg(),
            MappedAddress::TimerCounterReloadLowReg => self.timer.read_counter_reload_low_reg(),
            MappedAddress::TimerCounterReloadHighReg => self.timer.read_counter_reload_high_reg(),
            MappedAddress::TimerControlReg => self.timer.read_control_reg(),
            MappedAddress::WaitControlReg => {
                logln!("WARNING: Read byte from Wait Control Register not yet implemented");
                0
            }
            MappedAddress::GamePadInputControlReg => self.game_pad.read_input_control_reg(),
            MappedAddress::CartridgeExpansion(addr) => {
                logln!("WARNING: Read byte from Cartridge Expansion not yet implemented (addr: 0x{:08x})", addr);
                0
            }
            MappedAddress::Wram(addr) => self.wram.read_byte(addr),
            MappedAddress::CartridgeRam(addr) => self.sram.read_byte(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_byte(addr),
        }
    }

    pub fn read_halfword(&mut self, addr: u32) -> u16 {
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
            MappedAddress::GamePadInputLowReg => self.game_pad.read_input_low_reg() as _,
            MappedAddress::GamePadInputHighReg => self.game_pad.read_input_high_reg() as _,
            MappedAddress::TimerCounterReloadLowReg => self.timer.read_counter_reload_low_reg() as _,
            MappedAddress::TimerCounterReloadHighReg => self.timer.read_counter_reload_high_reg() as _,
            MappedAddress::TimerControlReg => self.timer.read_control_reg() as _,
            MappedAddress::WaitControlReg => {
                panic!("Read halfword from Wait Control Register not yet implemented");
            }
            MappedAddress::GamePadInputControlReg => self.game_pad.read_input_control_reg() as _,
            MappedAddress::CartridgeExpansion(addr) => {
                logln!("WARNING: Read halfword from Cartridge Expansion not yet implemented (addr: 0x{:08x})", addr);
                0
            }
            MappedAddress::Wram(addr) => self.wram.read_halfword(addr),
            MappedAddress::CartridgeRam(addr) => self.sram.read_halfword(addr),
            MappedAddress::CartridgeRom(addr) => self.rom.read_halfword(addr),
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match map_address(addr) {
            MappedAddress::Vip(addr) => self.vip.write_byte(addr, value),
            MappedAddress::Vsu(addr) => self.vsu.write_byte(addr, value),
            MappedAddress::LinkControlReg => {
                logln!("WARNING: Write byte to Link Control Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::AuxLinkReg => {
                logln!("WARNING: Write byte to Auxiliary Link Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::LinkTransmitDataReg => {
                logln!("WARNING: Write byte to Link Transmit Data Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::LinkReceiveDataReg => {
                logln!("WARNING: Write byte to Link Receive Data Register not yet implemented (value: 0x{:02x})", value);
            }
            MappedAddress::GamePadInputLowReg => {
                logln!("WARNING: Attempted write byte to Game Pad Input Low Register (value: 0x{:02x})", value);
            }
            MappedAddress::GamePadInputHighReg => {
                logln!("WARNING: Attempted write byte to Game Pad Input High Register (value: 0x{:02x})", value);
            }
            MappedAddress::TimerCounterReloadLowReg => self.timer.write_counter_reload_low_reg(value),
            MappedAddress::TimerCounterReloadHighReg => self.timer.write_counter_reload_high_reg(value),
            MappedAddress::TimerControlReg => self.timer.write_control_reg(value),
            MappedAddress::WaitControlReg => {
                logln!("Wait Control Register (0x{:08x}) written: 0x{:02x}", addr, value);
                logln!(" Cartridge ROM Waits: {}", if value & 0x01 == 0 { 2 } else { 1 });
                logln!(" Cartridge Expansion Waits: {}", if value & 0x02 == 0 { 2 } else { 1 });
            }
            MappedAddress::GamePadInputControlReg => self.game_pad.write_input_control_reg(value),
            MappedAddress::CartridgeExpansion(addr) => {
                logln!("WARNING: Write byte to Cartridge Expansion not yet implemented (addr: 0x{:08x}, value: 0x{:02x})", addr, value);
            }
            MappedAddress::Wram(addr) => self.wram.write_byte(addr, value),
            MappedAddress::CartridgeRam(addr) => self.sram.write_byte(addr, value),
            MappedAddress::CartridgeRom(_) => {
                logln!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr);
            }
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::Vip(addr) => self.vip.write_halfword(addr, value),
            MappedAddress::Vsu(addr) => self.vsu.write_halfword(addr, value),
            MappedAddress::LinkControlReg => {
                logln!("WARNING: Write halfword to Link Control Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::AuxLinkReg => {
                logln!("WARNING: Write halfword to Auxiliary Link Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::LinkTransmitDataReg => {
                logln!("WARNING: Write halfword to Link Transmit Data Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::LinkReceiveDataReg => {
                logln!("WARNING: Write halfword to Link Receive Data Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::GamePadInputLowReg => {
                logln!("WARNING: Attempted halfword byte to Game Pad Input Low Register (value: 0x{:04x})", value);
            }
            MappedAddress::GamePadInputHighReg => {
                logln!("WARNING: Attempted halfword byte to Game Pad Input High Register (value: 0x{:04x})", value);
            }
            MappedAddress::TimerCounterReloadLowReg => self.timer.write_counter_reload_low_reg(value as _),
            MappedAddress::TimerCounterReloadHighReg => self.timer.write_counter_reload_high_reg(value as _),
            MappedAddress::TimerControlReg => self.timer.write_control_reg(value as _),
            MappedAddress::WaitControlReg => {
                logln!("WARNING: Write halfword to Wait Control Register not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::GamePadInputControlReg => self.game_pad.write_input_control_reg(value as _),
            MappedAddress::CartridgeExpansion(addr) => {
                logln!("WARNING: Write halfword to Cartridge Expansion not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
            }
            MappedAddress::Wram(addr) => self.wram.write_halfword(addr, value),
            MappedAddress::CartridgeRam(addr) => self.sram.write_halfword(addr, value),
            MappedAddress::CartridgeRom(_) => {
                logln!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr);
            }
        }
    }

    pub fn cycles(&mut self, cycles: usize, video_driver: &mut VideoDriver) -> Option<u16> {
        let mut interrupt = None;

        if self.timer.cycles(cycles) {
            interrupt = Some(0xfe10);
        }

        if self.vip.cycles(cycles, video_driver) {
            interrupt = Some(0xfe40);
        }

        interrupt
    }
}
