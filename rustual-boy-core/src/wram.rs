pub const WRAM_SIZE: usize = 65536;

pub struct Wram {
    pub bytes: Box<[u8]>,
}

impl Wram {
    pub fn new() -> Wram {
        Wram {
            bytes: vec![0xff; WRAM_SIZE].into_boxed_slice(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Wram {
        Wram {
            bytes: bytes.to_vec().into_boxed_slice(),
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize] = value;
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
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

    fn mask_addr(&self, addr: u32) -> u32 {
        let mask = (WRAM_SIZE - 1) as u32;
        addr & mask
    }
}
