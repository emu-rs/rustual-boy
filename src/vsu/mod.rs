mod mem_map;

use audio_driver::*;
use self::mem_map::*;

pub struct Vsu {
    reg_sound_disable: bool,
}

impl Vsu {
    pub fn new() -> Vsu {
        Vsu {
            reg_sound_disable: false,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let halfword = self.read_halfword(addr & 0xfffffffe);
        if (addr & 0x01) == 0 {
            halfword as _
        } else {
            (halfword >> 8) as _
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let halfword = if (addr & 0x01) == 0 {
            value as _
        } else {
            (value as u16) << 8
        };
        self.write_halfword(addr & 0xfffffffe, halfword);
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        match addr {
            SOUND_DISABLE_REG => if self.reg_sound_disable { 1 } else { 0 },
            _ => {
                logln!("VSU read halfword not yet implemented (addr: 0x{:08x})", addr);
                0
            }
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        match addr {
            SOUND_DISABLE_REG => {
                self.reg_sound_disable = (value & 0x01) != 0;
            }
            _ => logln!("VSU write halfword not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value)
        }
    }

    pub fn cycles(&mut self, cycles: usize, audio_driver: &mut AudioDriver) {
        // TODO
    }
}