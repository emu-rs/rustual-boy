#![allow(dead_code)]

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
    reg_display_control_sync_enable: bool,

    reg_game_frame_control: usize,
}

impl Vip {
    pub fn new() -> Vip {
        Vip {
            vram: vec![0xff; VRAM_LENGTH as usize].into_boxed_slice(),

            display_state: DisplayState::Idle,

            reg_display_control_display_enable: false,
            reg_display_control_sync_enable: false,

            reg_game_frame_control: 1,
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
        match map_address(addr) {
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Read halfword from Interrupt Pending Reg not yet implemented");
                0
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Attempted read halfword from Interrupt Enable Reg");
                0
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Attempted read halfword from Interrupt Clear Reg");
                0
            }
            MappedAddress::DisplayControlReadReg => {
                let scan_ready = true; // TODO
                let frame_clock = false; // TODO
                let mem_refresh = false; // TODO
                let column_table_addr_lock = false; // TODO

                (if self.reg_display_control_display_enable { 1 } else { 0 } << 1) |
                (match self.display_state {
                    DisplayState::Idle => 0b0000,
                    DisplayState::LeftFramebufferDisplayProcessing => 0b0001, // TODO: Incorporate current framebuffer index
                    DisplayState::RightFramebufferDisplayProcessing => 0b0010, // TODO: Incorporate current framebuffer index
                } << 2) |
                (if scan_ready { 1 } else { 0 } << 6) |
                (if frame_clock { 1 } else { 0 } << 7) |
                (if mem_refresh { 1 } else { 0 } << 8) |
                (if self.reg_display_control_sync_enable { 1 } else { 0 } << 9) |
                (if column_table_addr_lock { 1 } else { 0 } << 10)
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted read halfword from Display Control Write Reg");
                0
            }
            MappedAddress::LedBrightness1Reg => {
                println!("WARNING: Read halfword from LED Brightness 1 Reg not yet implemented");
                0
            }
            MappedAddress::LedBrightness2Reg => {
                println!("WARNING: Read halfword from LED Brightness 2 Reg not yet implemented");
                0
            }
            MappedAddress::LedBrightness3Reg => {
                println!("WARNING: Read halfword from LED Brightness 3 Reg not yet implemented");
                0
            }
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Read halfword from LED Brightness Idle Reg not yet implemented");
                0
            }
            MappedAddress::GameFrameControlReg => {
                (self.reg_game_frame_control - 1) as u16
            }
            MappedAddress::DrawingControlReadReg => {
                println!("WARNING: Read halfword from Drawing Control Read Reg not yet implemented");
                0
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Attempted read halfword from Drawing Control Write Reg");
                0
            }
            MappedAddress::ObjGroup0PointerReg => {
                println!("WARNING: Read halfword from OBJ Group 0 Pointer Reg not yet implemented");
                0
            }
            MappedAddress::ObjGroup1PointerReg => {
                println!("WARNING: Read halfword from OBJ Group 1 Pointer Reg not yet implemented");
                0
            }
            MappedAddress::ObjGroup2PointerReg => {
                println!("WARNING: Read halfword from OBJ Group 2 Pointer Reg not yet implemented");
                0
            }
            MappedAddress::ObjGroup3PointerReg => {
                println!("WARNING: Read halfword from OBJ Group 3 Pointer Reg not yet implemented");
                0
            }
            MappedAddress::BgPalette0Reg => {
                println!("WARNING: Read halfword from BG Palette 0 Reg not yet implemented");
                0
            }
            MappedAddress::BgPalette1Reg => {
                println!("WARNING: Read halfword from BG Palette 1 Reg not yet implemented");
                0
            }
            MappedAddress::BgPalette2Reg => {
                println!("WARNING: Read halfword from BG Palette 2 Reg not yet implemented");
                0
            }
            MappedAddress::BgPalette3Reg => {
                println!("WARNING: Read halfword from BG Palette 3 Reg not yet implemented");
                0
            }
            MappedAddress::ObjPalette0Reg => {
                println!("WARNING: Read halfword from OBJ Palette 0 Reg not yet implemented");
                0
            }
            MappedAddress::ObjPalette1Reg => {
                println!("WARNING: Read halfword from OBJ Palette 1 Reg not yet implemented");
                0
            }
            MappedAddress::ObjPalette2Reg => {
                println!("WARNING: Read halfword from OBJ Palette 2 Reg not yet implemented");
                0
            }
            MappedAddress::ObjPalette3Reg => {
                println!("WARNING: Read halfword from OBJ Palette 3 Reg not yet implemented");
                0
            }
            MappedAddress::ClearColorReg => {
                println!("WARNING: Read halfword from Clear Color Reg not yet implemented");
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
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Attempted write halfword to Interrupt Pending Reg");
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Write halfword to Interrupt Enable Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Write halfword to Interrupt Clear Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted write halfword to Display Control Read Reg");
            }
            MappedAddress::DisplayControlWriteReg => {
                let _reset = (value & 0x01) != 0; // TODO: Soft reset
                self.reg_display_control_display_enable = (value & 0x02) != 0;
                let _mem_refresh = (value & 0x10) != 0; // TODO
                self.reg_display_control_sync_enable = (value & 0x20) != 0;
                let _column_table_addr_lock = (value & 0x40) != 0;

                // TODO
            }
            MappedAddress::LedBrightness1Reg => {
                println!("WARNING: Write halfword to LED Brightness 1 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::LedBrightness2Reg => {
                println!("WARNING: Write halfword to LED Brightness 2 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::LedBrightness3Reg => {
                println!("WARNING: Write halfword to LED Brightness 3 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Write halfword to LED Brightness Idle Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::GameFrameControlReg => {
                println!("Game Frame Control written (value: 0x{:04x})", value);
                self.reg_game_frame_control = (value as usize) + 1;
            }
            MappedAddress::DrawingControlReadReg => {
                println!("WARNING: Attempted write halfword to Drawing Control Read Reg (value: 0x{:04x})", value);
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Write halfword to Drawing Control Write Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjGroup0PointerReg => {
                println!("WARNING: Write halfword to OBJ Group 0 Pointer Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjGroup1PointerReg => {
                println!("WARNING: Write halfword to OBJ Group 1 Pointer Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjGroup2PointerReg => {
                println!("WARNING: Write halfword to OBJ Group 2 Pointer Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjGroup3PointerReg => {
                println!("WARNING: Write halfword to OBJ Group 3 Pointer Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::BgPalette0Reg => {
                println!("WARNING: Write halfword to BG Palette 0 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::BgPalette1Reg => {
                println!("WARNING: Write halfword to BG Palette 1 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::BgPalette2Reg => {
                println!("WARNING: Write halfword to BG Palette 2 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::BgPalette3Reg => {
                println!("WARNING: Write halfword to BG Palette 3 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjPalette0Reg => {
                println!("WARNING: Write halfword to OBJ Palette 0 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjPalette1Reg => {
                println!("WARNING: Write halfword to OBJ Palette 1 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjPalette2Reg => {
                println!("WARNING: Write halfword to OBJ Palette 2 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ObjPalette3Reg => {
                println!("WARNING: Write halfword to OBJ Palette 3 Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::ClearColorReg => {
                println!("WARNING: Write halfword to Clear Color Reg not yet implemented (value: 0x{:04x})", value);
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
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Attempted read word from Interrupt Pending Reg");
                0
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Attempted read word from Interrupt Enable Reg");
                0
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Attempted read word from Interrupt Clear Reg");
                0
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted read word from Display Control Read Reg");
                0
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted read word from Display Control Write Reg");
                0
            }
            MappedAddress::LedBrightness1Reg => {
                println!("WARNING: Attempted read word from LED Brightness 1 Reg");
                0
            }
            MappedAddress::LedBrightness2Reg => {
                println!("WARNING: Attempted read word from LED Brightness 2 Reg");
                0
            }
            MappedAddress::LedBrightness3Reg => {
                println!("WARNING: Attempted read word from LED Brightness 3 Reg");
                0
            }
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Attempted read word from LED Brightness Idle Reg");
                0
            }
            MappedAddress::GameFrameControlReg => {
                println!("WARNING: Attempted read word from Game Frame Control Reg");
                0
            }
            MappedAddress::DrawingControlReadReg => {
                println!("WARNING: Attempted read word from Drawing Control Read Reg");
                0
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Attempted read word from Drawing Control Write Reg");
                0
            }
            MappedAddress::ObjGroup0PointerReg => {
                println!("WARNING: Attempted read word from OBJ Group 0 Pointer Reg");
                0
            }
            MappedAddress::ObjGroup1PointerReg => {
                println!("WARNING: Attempted read word from OBJ Group 1 Pointer Reg");
                0
            }
            MappedAddress::ObjGroup2PointerReg => {
                println!("WARNING: Attempted read word from OBJ Group 2 Pointer Reg");
                0
            }
            MappedAddress::ObjGroup3PointerReg => {
                println!("WARNING: Attempted read word from OBJ Group 3 Pointer Reg");
                0
            }
            MappedAddress::BgPalette0Reg => {
                println!("WARNING: Attempted read word from BG Palette 0 Reg");
                0
            }
            MappedAddress::BgPalette1Reg => {
                println!("WARNING: Attempted read word from BG Palette 1 Reg");
                0
            }
            MappedAddress::BgPalette2Reg => {
                println!("WARNING: Attempted read word from BG Palette 2 Reg");
                0
            }
            MappedAddress::BgPalette3Reg => {
                println!("WARNING: Attempted read word from BG Palette 3 Reg");
                0
            }
            MappedAddress::ObjPalette0Reg => {
                println!("WARNING: Attempted read word from OBJ Palette 0 Reg");
                0
            }
            MappedAddress::ObjPalette1Reg => {
                println!("WARNING: Attempted read word from OBJ Palette 1 Reg");
                0
            }
            MappedAddress::ObjPalette2Reg => {
                println!("WARNING: Attempted read word from OBJ Palette 2 Reg");
                0
            }
            MappedAddress::ObjPalette3Reg => {
                println!("WARNING: Attempted read word from OBJ Palette 3 Reg");
                0
            }
            MappedAddress::ClearColorReg => {
                println!("WARNING: Attempted read word from Clear Color Reg");
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
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Attempted write word to Interrupt Pending Reg (value: 0x{:08x})", value);
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Attempted write word to Interrupt Enable Reg (value: 0x{:08x})", value);
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Attempted write word to Interrupt Clear Reg (value: 0x{:08x})", value);
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted write word to Display Control Read Reg (value: 0x{:08x})", value);
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted write word to Display Control Write Reg (value: 0x{:08x})", value);
            }
            MappedAddress::LedBrightness1Reg => {
                println!("WARNING: Attempted write word to LED Brightness 1 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::LedBrightness2Reg => {
                println!("WARNING: Attempted write word to LED Brightness 2 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::LedBrightness3Reg => {
                println!("WARNING: Attempted write word to LED Brightness 3 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Attempted write word to LED Brightness Idle Reg (value: 0x{:08x})", value);
            }
            MappedAddress::GameFrameControlReg => {
                println!("WARNING: Attempted write word to Game Frame Control Reg (value: 0x{:08x})", value);
            }
            MappedAddress::DrawingControlReadReg => {
                println!("WARNING: Attempted write word to Drawing Control Read Reg (value: 0x{:08x})", value);
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Attempted write word to Drawing Control Write Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjGroup0PointerReg => {
                println!("WARNING: Attempted write word to OBJ Group 0 Pointer Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjGroup1PointerReg => {
                println!("WARNING: Attempted write word to OBJ Group 1 Pointer Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjGroup2PointerReg => {
                println!("WARNING: Attempted write word to OBJ Group 2 Pointer Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjGroup3PointerReg => {
                println!("WARNING: Attempted write word to OBJ Group 3 Pointer Reg (value: 0x{:08x})", value);
            }
            MappedAddress::BgPalette0Reg => {
                println!("WARNING: Attempted write word to BG Palette 0 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::BgPalette1Reg => {
                println!("WARNING: Attempted write word to BG Palette 1 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::BgPalette2Reg => {
                println!("WARNING: Attempted write word to BG Palette 2 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::BgPalette3Reg => {
                println!("WARNING: Attempted write word to BG Palette 3 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjPalette0Reg => {
                println!("WARNING: Attempted write word to OBJ Palette 0 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjPalette1Reg => {
                println!("WARNING: Attempted write word to OBJ Palette 1 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjPalette2Reg => {
                println!("WARNING: Attempted write word to OBJ Palette 2 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ObjPalette3Reg => {
                println!("WARNING: Attempted write word to OBJ Palette 3 Reg (value: 0x{:08x})", value);
            }
            MappedAddress::ClearColorReg => {
                println!("WARNING: Attempted write word to Clear Color Reg (value: 0x{:08x})", value);
            }
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value as u8;
                self.vram[addr as usize + 1] = (value >> 8) as u8;
                self.vram[addr as usize + 2] = (value >> 16) as u8;
                self.vram[addr as usize + 3] = (value >> 24) as u8;
            }
        }
    }

    pub fn cycles(&mut self, _cycles: usize) {
        // TODO
    }
}