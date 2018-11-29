use std::io;
use std::fs::{read, write};
use std::path::Path;

pub const MIN_SRAM_SIZE: usize = 1024;
pub const MAX_SRAM_SIZE: usize = 16 * 1024 * 1024;

// TODO: Proper display impl
#[derive(Debug)]
pub enum SramError {
    InvalidSize,
    Io(io::Error),
}

#[derive(Clone)]
pub struct Sram {
    pub bytes: Box<[u8]>,

    pub size: usize,
}

impl Sram {
    pub fn new() -> Sram {
        Sram {
            bytes: vec![0xff; MAX_SRAM_SIZE].into_boxed_slice(),

            size: 0,
        }
    }

    pub fn load<P: AsRef<Path>>(file_name: P) -> Result<Sram, SramError> {
        let file = read(file_name).map_err(|e| SramError::Io(e))?;

        let size = file.len();
        if size < MIN_SRAM_SIZE || size > MAX_SRAM_SIZE || !size.is_power_of_two() {
            return Err(SramError::InvalidSize);
        }

        let mut bytes = vec![0xff; MAX_SRAM_SIZE].into_boxed_slice();
        bytes[..size].copy_from_slice(&file);

        Ok(Sram {
            bytes,

            size,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, file_name: P) -> io::Result<()> {
        write(file_name, &self.bytes[..self.size])
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
        ((self.bytes[(addr + 1) as usize] as u16) << 8)
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize] = value as _;
        self.bytes[(addr + 1) as usize] = (value >> 8) as _;
    }

    fn mask_addr(&mut self, addr: u32) -> u32 {
        let mask = (MAX_SRAM_SIZE - 1) as u32;
        let addr = addr & mask;
        while addr >= self.size as u32 {
            self.size = match self.size {
                0 => MIN_SRAM_SIZE,
                _ => self.size * 2,
            };
        }
        addr
    }
}
