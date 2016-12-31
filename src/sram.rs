use std::io::{self, Write};
use std::fs::File;
use std::path::Path;

pub const MIN_SRAM_SIZE: usize = 1024;
pub const MAX_SRAM_SIZE: usize = 16 * 1024 * 1024;

pub struct Sram {
    bytes: Box<[u8]>,
    size: usize,
}

impl Sram {
    pub fn new() -> Sram {
        Sram {
            bytes: vec![0xff; MAX_SRAM_SIZE].into_boxed_slice(),
            size: 0,
        }
    }

    pub fn save<P: AsRef<Path>>(&self, file_name: P) -> io::Result<()> {
        let mut file = File::create(file_name)?;
        file.write_all(&self.bytes)
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn read_byte(&mut self, addr: u32) -> u8 {
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize] = value;
    }

    pub fn read_halfword(&mut self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        let addr = self.mask_addr(addr);
        (self.bytes[addr as usize] as u16) |
        ((self.bytes[addr as usize + 1] as u16) << 8)
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize] = value as u8;
        self.bytes[addr as usize + 1] = (value >> 8) as u8;
    }

    pub fn read_word(&mut self, addr: u32) -> u32 {
        let addr = addr & 0xfffffffc;
        let addr = self.mask_addr(addr);
        (self.bytes[addr as usize] as u32) |
        ((self.bytes[addr as usize + 1] as u32) << 8) |
        ((self.bytes[addr as usize + 2] as u32) << 16) |
        ((self.bytes[addr as usize + 3] as u32) << 24)
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        let addr = addr & 0xfffffffc;
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize] = value as u8;
        self.bytes[addr as usize + 1] = (value >> 8) as u8;
        self.bytes[addr as usize + 2] = (value >> 16) as u8;
        self.bytes[addr as usize + 3] = (value >> 24) as u8;
    }

    fn mask_addr(&mut self, addr: u32) -> u32 {
        let mask = (MAX_SRAM_SIZE - 1) as u32;
        let addr = addr & mask;
        if addr >= self.size as u32 {
            self.size = match self.size {
                0 => MIN_SRAM_SIZE,
                _ => self.size * 2,
            };
        }
        addr
    }
}
