use com_port::*;
use game_pad::*;
use mem_map::*;
use rom::*;
use sinks::*;
use sram::*;
use timer::*;
use vip::*;
use vsu::*;
use wram::*;

pub struct Interconnect {
    pub rom: Rom,
    pub wram: Wram,
    pub sram: Sram,
    pub vip: Vip,
    pub vsu: Vsu,
    pub timer: Timer,
    pub game_pad: GamePad,
    pub com_port: ComPort,
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
            com_port: ComPort::new(),
        }
    }

    pub fn read_byte(&mut self, addr: u32) -> u8 {
        let addr = addr & 0x07ffffff;
        match addr {
            VIP_START ... VIP_END => self.vip.read_byte(addr - VIP_START),
            VSU_START ... VSU_END => self.vsu.read_byte(addr - VSU_START),
            CCR => self.com_port.read_ccr(),
            CCSR => self.com_port.read_ccsr(),
            CDTR => self.com_port.read_cdtr(),
            CDRR => self.com_port.read_cdrr(),
            SDLR => self.game_pad.read_sdlr(),
            SDHR => self.game_pad.read_sdhr(),
            TLR => self.timer.read_tlr(),
            THR => self.timer.read_thr(),
            TCR => self.timer.read_tcr(),
            WCR => {
                logln!(Log::Ic, "WARNING: Read byte from WCR not yet implemented");
                0
            }
            SCR => self.game_pad.read_scr(),
            GAME_PAK_EXPANSION_START ... GAME_PAK_EXPANSION_END => {
                logln!(Log::Ic, "WARNING: Read byte from Game Pak Expansion not yet implemented (addr: 0x{:08x})", addr - GAME_PAK_EXPANSION_START);
                0
            }
            WRAM_START ... WRAM_END => self.wram.read_byte(addr - WRAM_START),
            GAME_PAK_RAM_START ... GAME_PAK_RAM_END => self.sram.read_byte(addr - GAME_PAK_RAM_START),
            GAME_PAK_ROM_START ... GAME_PAK_ROM_END => self.rom.read_byte(addr - GAME_PAK_ROM_START),
            _ => panic!("Unrecognized addr: 0x{:08x}", addr)
        }
    }

    pub fn read_halfword(&mut self, addr: u32) -> u16 {
        let addr = addr & 0x07ffffff;
        let addr = addr & 0xfffffffe;
        match addr {
            VIP_START ... VIP_END => self.vip.read_halfword(addr - VIP_START),
            VSU_START ... VSU_END => self.vsu.read_halfword(addr - VSU_START),
            CCR => self.com_port.read_ccr() as _,
            CCSR => self.com_port.read_ccsr() as _,
            CDTR => self.com_port.read_cdtr() as _,
            CDRR => self.com_port.read_cdrr() as _,
            SDLR => self.game_pad.read_sdlr() as _,
            SDHR => self.game_pad.read_sdhr() as _,
            TLR => self.timer.read_tlr() as _,
            THR => self.timer.read_thr() as _,
            TCR => self.timer.read_tcr() as _,
            WCR => {
                logln!(Log::Ic, "Read halfword from WCR not yet implemented");
                0
            }
            SCR => self.game_pad.read_scr() as _,
            GAME_PAK_EXPANSION_START ... GAME_PAK_EXPANSION_END => {
                logln!(Log::Ic, "WARNING: Read halfword from Game Pak Expansion not yet implemented (addr: 0x{:08x})", addr - GAME_PAK_EXPANSION_START);
                0
            }
            WRAM_START ... WRAM_END => self.wram.read_halfword(addr - WRAM_START),
            GAME_PAK_RAM_START ... GAME_PAK_RAM_END => self.sram.read_halfword(addr - GAME_PAK_RAM_START),
            GAME_PAK_ROM_START ... GAME_PAK_ROM_END => self.rom.read_halfword(addr - GAME_PAK_ROM_START),
            _ => panic!("Unrecognized addr: 0x{:08x}", addr)
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = addr & 0x07ffffff;
        match addr {
            VIP_START ... VIP_END => self.vip.write_byte(addr - VIP_START, value),
            VSU_START ... VSU_END => self.vsu.write_byte(addr - VSU_START, value),
            CCR => self.com_port.write_ccr(value),
            CCSR => self.com_port.write_ccsr(value),
            CDTR => self.com_port.write_cdtr(value),
            CDRR => {
                logln!(Log::Ic, "WARNING: Attempted write byte to CDRR (value: 0x{:02x})", value);
            }
            SDLR => {
                logln!(Log::Ic, "WARNING: Attempted write byte to SDLR (value: 0x{:02x})", value);
            }
            SDHR => {
                logln!(Log::Ic, "WARNING: Attempted write byte to SDHR (value: 0x{:02x})", value);
            }
            TLR => self.timer.write_tlr(value),
            THR => self.timer.write_thr(value),
            TCR => self.timer.write_tcr(value),
            WCR => {
                logln!(Log::Ic, "WCR (0x{:08x}) written: 0x{:02x}", addr, value);
                logln!(Log::Ic, " Game Pak ROM Waits: {}", if value & 0x01 == 0 { 2 } else { 1 });
                logln!(Log::Ic, " Game Pak Expansion Waits: {}", if value & 0x02 == 0 { 2 } else { 1 });
            }
            SCR => self.game_pad.write_scr(value),
            GAME_PAK_EXPANSION_START ... GAME_PAK_EXPANSION_END => {
                logln!(Log::Ic, "WARNING: Write byte to Game Pak Expansion not yet implemented (addr: 0x{:08x}, value: 0x{:02x})", addr - GAME_PAK_EXPANSION_START, value);
            }
            WRAM_START ... WRAM_END => self.wram.write_byte(addr - WRAM_START, value),
            GAME_PAK_RAM_START ... GAME_PAK_RAM_END => self.sram.write_byte(addr - GAME_PAK_RAM_START, value),
            GAME_PAK_ROM_START ... GAME_PAK_ROM_END => {
                logln!(Log::Ic, "WARNING: Attempted write to Game Pak ROM at 0x{:08x}", addr - GAME_PAK_ROM_START);
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
            CCR => self.com_port.write_ccr(value as _),
            CCSR => self.com_port.write_ccsr(value as _),
            CDTR => self.com_port.write_cdtr(value as _),
            CDRR => {
                logln!(Log::Ic, "WARNING: Attempted write halfword to CDRR (value: 0x{:04x})", value);
            }
            SDLR => {
                logln!(Log::Ic, "WARNING: Attempted write halfword byte to SDLR (value: 0x{:04x})", value);
            }
            SDHR => {
                logln!(Log::Ic, "WARNING: Attempted write halfword byte to SDHR (value: 0x{:04x})", value);
            }
            TLR => self.timer.write_tlr(value as _),
            THR => self.timer.write_thr(value as _),
            TCR => self.timer.write_tcr(value as _),
            WCR => {
                logln!(Log::Ic, "WARNING: Write halfword to WCR not yet implemented (value: 0x{:04x})", value);
            }
            SCR => self.game_pad.write_scr(value as _),
            GAME_PAK_EXPANSION_START ... GAME_PAK_EXPANSION_END => {
                logln!(Log::Ic, "WARNING: Write halfword to Game Pak Expansion not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr - GAME_PAK_EXPANSION_START, value);
            }
            WRAM_START ... WRAM_END => self.wram.write_halfword(addr - WRAM_START, value),
            GAME_PAK_RAM_START ... GAME_PAK_RAM_END => self.sram.write_halfword(addr - GAME_PAK_RAM_START, value),
            GAME_PAK_ROM_START ... GAME_PAK_ROM_END => {
                logln!(Log::Ic, "WARNING: Attempted write to Game Pak ROM at 0x{:08x}", addr - GAME_PAK_ROM_START);
            }
            _ => panic!("Unrecognized addr: 0x{:08x}", addr)
        }
    }

    pub fn cycles(&mut self, cycles: u32, video_frame_sink: &mut VideoSink, audio_frame_sink: &mut AudioSink) -> Option<u16> {
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
