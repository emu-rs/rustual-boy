mod mem_map;

use self::mem_map::*;

pub struct Vip {
    vram: Box<[u8]>,
}

impl Vip {
    pub fn new() -> Vip {
        Vip {
            vram: vec![0; VRAM_LENGTH as usize].into_boxed_slice(),
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        panic!("VIP read byte not yet implemented (addr: 0x{:08x})", addr);
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        panic!("VIP write byte not yet implemented (addr: 0x{:08x}, value: 0x{:02x})", addr, value);
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        panic!("VIP read halfword not yet implemented (addr: 0x{:08x})", addr);
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        panic!("VIP write halfword not yet implemented (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
    }

    pub fn read_word(&self, addr: u32) -> u32 {
        let addr = addr & 0xfffffffc;
        panic!("VIP read word not yet implemented (addr: 0x{:08x})", addr);
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        let addr = addr & 0xfffffffc;
        match map_address(addr) {
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value as u8;
                self.vram[addr as usize + 1] = (value >> 8) as u8;
                self.vram[addr as usize + 2] = (value >> 16) as u8;
                self.vram[addr as usize + 3] = (value >> 24) as u8;
            }
        }
    }
}