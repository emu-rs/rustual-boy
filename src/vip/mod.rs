mod mem_map;

use self::mem_map::*;

const MS_TO_NS: u64 = 1000000;

const CPU_CYCLE_PERIOD_NS: u64 = 50;
const FRAME_CLOCK_PERIOD_MS: u64 = 20;
const FRAME_CLOCK_PERIOD_NS: u64 = FRAME_CLOCK_PERIOD_MS * MS_TO_NS;

enum DisplayState {
    Idle,
    LeftFramebufferDisplayProcessing,
    RightFramebufferDisplayProcessing,
}

pub struct Vip {
    vram: Box<[u8]>,

    display_state: DisplayState,

    reg_display_control_display_enable: bool,

    reg_game_frame_control: usize,
}

impl Vip {
    pub fn new() -> Vip {
        Vip {
            vram: vec![0xff; VRAM_LENGTH as usize].into_boxed_slice(),

            display_state: DisplayState::Idle,

            reg_display_control_display_enable: false,

            reg_game_frame_control: 0,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        panic!("VIP read byte not yet implemented (addr: 0x{:08x})", addr);
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        panic!("VIP write byte not yet implemented (addr: 0x{:08x}, value: 0x{:02x})", addr, value);
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffc;
        match map_address(addr) {
            MappedAddress::DisplayControlReadReg => {
                let scan_ready = true; // TODO
                let frame_clock = false; // TODO
                let refresh = false; // TODO
                let sync_enable = false; // TODO
                let columb_table_addr_lock = false; // TODO

                (if self.reg_display_control_display_enable { 1 } else { 0 } << 1) |
                (match self.display_state {
                    DisplayState::Idle => 0b0000,
                    DisplayState::LeftFramebufferDisplayProcessing => 0b0001, // TODO: Incorporate current framebuffer index
                    DisplayState::RightFramebufferDisplayProcessing => 0b0010, // TODO: Incorporate current framebuffer index
                } << 2) |
                (if scan_ready { 1 } else { 0 } << 6) |
                (if frame_clock { 1 } else { 0 } << 7) |
                (if refresh { 1 } else { 0 } << 8) |
                (if sync_enable { 1 } else { 0 } << 9) |
                (if columb_table_addr_lock { 1 } else { 0 } << 10)
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted read halfword from Display Control Write Reg");
                0
            }
            MappedAddress::Vram(addr) => {
                (self.vram[addr as usize] as u16) |
                ((self.vram[addr as usize + 1] as u16) << 8)
            }
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted write halfword to Display Control Read Reg");
            }
            MappedAddress::DisplayControlWriteReg => {
                panic!("WARNING: Attempted write halfword to Display Control Write Reg (value: 0x{:04x})", value);
            }
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value as u8;
                self.vram[addr as usize + 1] = (value >> 8) as u8;
            }
        }
    }

    pub fn read_word(&self, addr: u32) -> u32 {
        let addr = addr & 0xfffffffc;
        match map_address(addr) {
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted read word from Display Control Read Reg");
                0
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted read word from Display Control Write Reg");
                0
            }
            MappedAddress::Vram(addr) => {
                (self.vram[addr as usize] as u32) |
                ((self.vram[addr as usize + 1] as u32) << 8) |
                ((self.vram[addr as usize + 2] as u32) << 16) |
                ((self.vram[addr as usize + 3] as u32) << 24)
            }
        }
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        let addr = addr & 0xfffffffc;
        match map_address(addr) {
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted write word to Display Control Read Reg (value: 0x{:08x})", value);
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted write word to Display Control Write Reg (value: 0x{:08x})", value);
            }
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value as u8;
                self.vram[addr as usize + 1] = (value >> 8) as u8;
                self.vram[addr as usize + 2] = (value >> 16) as u8;
                self.vram[addr as usize + 3] = (value >> 24) as u8;
            }
        }
    }

    pub fn cycles(&mut self, cycles: usize) {
    }
}