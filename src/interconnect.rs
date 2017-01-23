use video_frame_sink::*;
use audio_frame_sink::*;
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
        let addr = addr & 0x07ffffff;
        match addr {
            VIP_START ... VIP_END => self.vip.read_byte(addr - VIP_START),
            VSU_START ... VSU_END => self.vsu.read_byte(addr - VSU_START),
            LINK_CONTROL_REG => {
                logln!("WARNING: Read byte from Link Control Register not yet implemented");
                0
            }
            AUX_LINK_REG => {
                logln!("WARNING: Read byte from Auxiliary Link Register not yet implemented");
                0
            }
            LINK_TRANSMIT_DATA_REG => {
                logln!("Read byte from Link Transmit Data Register not yet implemented");
                0
            }
            LINK_RECEIVE_DATA_REG => {
                logln!("Read byte from Link Receive Data Register not yet implemented");
                0
            }
            GAME_PAD_INPUT_LOW_REG => self.game_pad.read_input_low_reg(),
            GAME_PAD_INPUT_HIGH_REG => self.game_pad.read_input_high_reg(),
            TIMER_COUNTER_RELOAD_LOW_REG => self.timer.read_counter_reload_low_reg(),
            TIMER_COUNTER_RELOAD_HIGH_REG => self.timer.read_counter_reload_high_reg(),
            TIMER_CONTROL_REG => self.timer.read_control_reg(),
            WAIT_CONTROL_REG => {
                logln!("WARNING: Read byte from Wait Control Register not yet implemented");
                0
            }
            GAME_PAD_INPUT_CONTROL_REG => self.game_pad.read_input_control_reg(),
            CARTRIDGE_EXPANSION_START ... CARTRIDGE_EXPANSION_END => {
                logln!("WARNING: Read byte from Cartridge Expansion not yet implemented (addr: 0x{:08x})", addr - CARTRIDGE_EXPANSION_START);
                0
            }
            WRAM_START ... WRAM_END => self.wram.read_byte(addr - WRAM_START),
            CARTRIDGE_RAM_START ... CARTRIDGE_RAM_END => self.sram.read_byte(addr - CARTRIDGE_RAM_START),
            CARTRIDGE_ROM_START ... CARTRIDGE_ROM_END => self.rom.read_byte(addr - CARTRIDGE_ROM_START),
            _ => panic!("Unrecognized addr: 0x{:08x}", addr)
        }
    }

    pub fn read_halfword(&mut self, addr: u32) -> u16 {
        let addr = addr & 0x07ffffff;
        let addr = addr & 0xfffffffe;
        match addr {
            VIP_START ... VIP_END => self.vip.read_halfword(addr - VIP_START),
            VSU_START ... VSU_END => self.vsu.read_halfword(addr - VSU_START),
            LINK_CONTROL_REG => {
                logln!("Read halfword from Link Control Register not yet implemented");
                0
            }
            AUX_LINK_REG => {
                logln!("Read halfword from Auxiliary Link Register not yet implemented");
                0
            }
            LINK_TRANSMIT_DATA_REG => {
                logln!("Read halfword from Link Transmit Data Register not yet implemented");
                0
            }
            LINK_RECEIVE_DATA_REG => {
                logln!("Read halfword from Link Receive Data Register not yet implemented");
                0
            }
            GAME_PAD_INPUT_LOW_REG => self.game_pad.read_input_low_reg() as _,
            GAME_PAD_INPUT_HIGH_REG => self.game_pad.read_input_high_reg() as _,
            TIMER_COUNTER_RELOAD_LOW_REG => self.timer.read_counter_reload_low_reg() as _,
            TIMER_COUNTER_RELOAD_HIGH_REG => self.timer.read_counter_reload_high_reg() as _,
            TIMER_CONTROL_REG => self.timer.read_control_reg() as _,
            WAIT_CONTROL_REG => {
                logln!("Read halfword from Wait Control Register not yet implemented");
                0
            }
            GAME_PAD_INPUT_CONTROL_REG => self.game_pad.read_input_control_reg() as _,
            CARTRIDGE_EXPANSION_START ... CARTRIDGE_EXPANSION_END => {
                logln!("WARNING: Read halfword from Cartridge Expansion not yet implemented (addr: 0x{:08x})", addr - CARTRIDGE_EXPANSION_START);
                0
            }
            WRAM_START ... WRAM_END => self.wram.read_halfword(addr - WRAM_START),
            CARTRIDGE_RAM_START ... CARTRIDGE_RAM_END => self.sram.read_halfword(addr - CARTRIDGE_RAM_START),
            CARTRIDGE_ROM_START ... CARTRIDGE_ROM_END => self.rom.read_halfword(addr - CARTRIDGE_ROM_START),
            _ => panic!("Unrecognized addr: 0x{:08x}", addr)
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = addr & 0x07ffffff;
        match addr {
            VIP_START ... VIP_END => self.vip.write_byte(addr - VIP_START, value),
            VSU_START ... VSU_END => self.vsu.write_byte(addr - VSU_START, value),
            LINK_CONTROL_REG => {
                logln!("WARNING: Write byte to Link Control Register not yet implemented (value: 0x{:02x})", value);
            }
            AUX_LINK_REG => {
                logln!("WARNING: Write byte to Auxiliary Link Register not yet implemented (value: 0x{:02x})", value);
            }
            LINK_TRANSMIT_DATA_REG => {
                logln!("WARNING: Write byte to Link Transmit Data Register not yet implemented (value: 0x{:02x})", value);
            }
            LINK_RECEIVE_DATA_REG => {
                logln!("WARNING: Write byte to Link Receive Data Register not yet implemented (value: 0x{:02x})", value);
            }
            GAME_PAD_INPUT_LOW_REG => {
                logln!("WARNING: Attempted write byte to Game Pad Input Low Register (value: 0x{:02x})", value);
            }
            GAME_PAD_INPUT_HIGH_REG => {
                logln!("WARNING: Attempted write byte to Game Pad Input High Register (value: 0x{:02x})", value);
            }
            TIMER_COUNTER_RELOAD_LOW_REG => self.timer.write_counter_reload_low_reg(value),
            TIMER_COUNTER_RELOAD_HIGH_REG => self.timer.write_counter_reload_high_reg(value),
            TIMER_CONTROL_REG => self.timer.write_control_reg(value),
            WAIT_CONTROL_REG => {
                logln!("Wait Control Register (0x{:08x}) written: 0x{:02x}", addr, value);
                logln!(" Cartridge ROM Waits: {}", if value & 0x01 == 0 { 2 } else { 1 });
                logln!(" Cartridge Expansion Waits: {}", if value & 0x02 == 0 { 2 } else { 1 });
            }
            GAME_PAD_INPUT_CONTROL_REG => self.game_pad.write_input_control_reg(value),
            CARTRIDGE_EXPANSION_START ... CARTRIDGE_EXPANSION_END => {
                logln!("WARNING: Write byte to Cartridge Expansion not yet implemented (addr: 0x{:08x}, value: 0x{:02x})", addr - CARTRIDGE_EXPANSION_START, value);
            }
            WRAM_START ... WRAM_END => self.wram.write_byte(addr - WRAM_START, value),
            CARTRIDGE_RAM_START ... CARTRIDGE_RAM_END => self.sram.write_byte(addr - CARTRIDGE_RAM_START, value),
            CARTRIDGE_ROM_START ... CARTRIDGE_ROM_END => {
                logln!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr - CARTRIDGE_ROM_START);
            }
            _ => panic!("Unrecognized addr: 0x{:08x}", addr)
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0x07ffffff;
        let addr = addr & 0xfffffffe;
        match addr {
            VIP_START ... VIP_END => self.vip.write_halfword(addr - VIP_START, value),
            VSU_START ... VSU_END => self.vsu.write_halfword(addr - VSU_START, value),
            LINK_CONTROL_REG => {
                logln!("WARNING: Write halfword to Link Control Register not yet implemented (value: 0x{:04x})", value);
            }
            AUX_LINK_REG => {
                logln!("WARNING: Write halfword to Auxiliary Link Register not yet implemented (value: 0x{:04x})", value);
            }
            LINK_TRANSMIT_DATA_REG => {
                logln!("WARNING: Write halfword to Link Transmit Data Register not yet implemented (value: 0x{:04x})", value);
            }
            LINK_RECEIVE_DATA_REG => {
                logln!("WARNING: Write halfword to Link Receive Data Register not yet implemented (value: 0x{:04x})", value);
            }
            GAME_PAD_INPUT_LOW_REG => {
                logln!("WARNING: Attempted halfword byte to Game Pad Input Low Register (value: 0x{:04x})", value);
            }
            GAME_PAD_INPUT_HIGH_REG => {
                logln!("WARNING: Attempted halfword byte to Game Pad Input High Register (value: 0x{:04x})", value);
            }
            TIMER_COUNTER_RELOAD_LOW_REG => self.timer.write_counter_reload_low_reg(value as _),
            TIMER_COUNTER_RELOAD_HIGH_REG => self.timer.write_counter_reload_high_reg(value as _),
            TIMER_CONTROL_REG => self.timer.write_control_reg(value as _),
            WAIT_CONTROL_REG => {
                logln!("WARNING: Write halfword to Wait Control Register not yet implemented (value: 0x{:04x})", value);
            }
            GAME_PAD_INPUT_CONTROL_REG => self.game_pad.write_input_control_reg(value as _),
            CARTRIDGE_EXPANSION_START ... CARTRIDGE_EXPANSION_END => {
                logln!("WARNING: Write halfword to Cartridge Expansion not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr - CARTRIDGE_EXPANSION_START, value);
            }
            WRAM_START ... WRAM_END => self.wram.write_halfword(addr - WRAM_START, value),
            CARTRIDGE_RAM_START ... CARTRIDGE_RAM_END => self.sram.write_halfword(addr - CARTRIDGE_RAM_START, value),
            CARTRIDGE_ROM_START ... CARTRIDGE_ROM_END => {
                logln!("WARNING: Attempted write to Cartridge ROM at 0x{:08x}", addr - CARTRIDGE_ROM_START);
            }
            _ => panic!("Unrecognized addr: 0x{:08x}", addr)
        }
    }

    pub fn cycles(&mut self, cycles: usize, video_frame_sink: &mut VideoFrameSink, audio_frame_sink: &mut AudioFrameSink) -> Option<u16> {
        let mut interrupt = None;

        if self.timer.cycles(cycles) {
            interrupt = Some(0xfe10);
        }

        if self.vip.cycles(cycles, video_frame_sink) {
            interrupt = Some(0xfe40);
        }

        self.vsu.cycles(cycles, audio_frame_sink);

        interrupt
    }
}
