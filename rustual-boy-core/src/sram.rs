use std::io::{self, Read, Write, Error, ErrorKind};
use std::fs::File;
use std::path::Path;

pub const MIN_SRAM_SIZE: usize = 1024;
pub const MAX_SRAM_SIZE: usize = 16 * 1024 * 1024;

pub struct Sram {
    bytes: Box<[u8]>,
    bytes_ptr: *mut u8,

    size: usize,
}

impl Sram {
    pub fn new() -> Sram {
        let mut bytes = vec![0xff; MAX_SRAM_SIZE].into_boxed_slice();
        let bytes_ptr = bytes.as_mut_ptr();

        Sram {
            bytes: bytes,
            bytes_ptr: bytes_ptr,

            size: 0,
        }
    }

    pub fn load<P: AsRef<Path>>(file_name: P) -> io::Result<Sram> {
        let mut file = File::open(file_name)?;
        let mut vec = Vec::new();
        file.read_to_end(&mut vec)?;

        let size = vec.len();
        if size < MIN_SRAM_SIZE || size > MAX_SRAM_SIZE || !size.is_power_of_two() {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid SRAM size"));
        }

        let mut bytes = vec.into_boxed_slice();
        let bytes_ptr = bytes.as_mut_ptr();

        Ok(Sram {
            bytes: bytes,
            bytes_ptr: bytes_ptr,

            size: size,
        })
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
        unsafe {
            *self.bytes_ptr.offset(addr as _)
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = self.mask_addr(addr);
        unsafe {
            *self.bytes_ptr.offset(addr as _) = value;
        }
    }

    pub fn read_halfword(&mut self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        let addr = self.mask_addr(addr);
        unsafe {
            (*self.bytes_ptr.offset(addr as _) as u16) |
            ((*self.bytes_ptr.offset((addr + 1) as _) as u16) << 8)
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        let addr = self.mask_addr(addr);
        unsafe {
            *self.bytes_ptr.offset(addr as _) = value as _;
            *self.bytes_ptr.offset((addr + 1) as _) = (value >> 8) as _;
        }
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

impl Clone for Sram {
    fn clone(&self) -> Sram {
        let mut bytes_box = self.bytes.clone();
        let bytes_ptr = bytes_box.as_mut_ptr();

        Sram {
            bytes: bytes_box,
            bytes_ptr: bytes_ptr,

            size: self.size,
        }
    }
}
