pub const WRAM_SIZE: usize = 65536;

pub struct Wram {
    _bytes: Box<[u8]>,
    bytes_ptr: *mut u8,
}

impl Wram {
    pub fn new() -> Wram {
        let mut bytes = vec![0xff; WRAM_SIZE].into_boxed_slice();
        let bytes_ptr = bytes.as_mut_ptr();

        Wram {
            _bytes: bytes,
            bytes_ptr: bytes_ptr,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
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

    pub fn read_halfword(&self, addr: u32) -> u16 {
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

    fn mask_addr(&self, addr: u32) -> u32 {
        let mask = (WRAM_SIZE - 1) as u32;
        addr & mask
    }
}