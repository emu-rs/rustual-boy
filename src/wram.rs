pub const WRAM_SIZE: usize = 65536;

pub struct Wram {
    bytes: Box<[u8]>,
}

impl Wram {
    pub fn new() -> Wram {
        Wram {
            bytes: vec![0xff; WRAM_SIZE].into_boxed_slice(),
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
        ((self.bytes[addr as usize + 1] as u16) << 8)
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize] = value as u8;
        self.bytes[addr as usize + 1] = (value >> 8) as u8;
    }

    pub fn read_word(&self, addr: u32) -> u32 {
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

    fn mask_addr(&self, addr: u32) -> u32 {
        let mask = (WRAM_SIZE - 1) as u32;
        addr & mask
    }
}