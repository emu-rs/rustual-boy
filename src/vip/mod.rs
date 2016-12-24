#![allow(dead_code)]

mod mem_map;

use video_driver::*;
use self::mem_map::*;

const MS_TO_NS: u64 = 1000000;

const CPU_CYCLE_PERIOD_NS: u64 = 50;

const FRAME_CLOCK_PERIOD_MS: u64 = 20;
const FRAME_CLOCK_PERIOD_NS: u64 = FRAME_CLOCK_PERIOD_MS * MS_TO_NS;

// Hardcoded drawing period for now
const DRAWING_PERIOD_MS: u64 = 10;
const DRAWING_PERIOD_NS: u64 = DRAWING_PERIOD_MS * MS_TO_NS;

enum DisplayState {
    Idle,
    LeftFramebufferDisplayProcessing,
    RightFramebufferDisplayProcessing,
}

enum DrawingState {
    Idle,
    Drawing,
}

pub struct Vip {
    vram: Box<[u8]>,

    display_state: DisplayState,

    drawing_state: DrawingState,

    reg_interrupt_pending_drawing_started: bool,
    reg_interrupt_pending_start_of_frame_processing: bool,
    reg_interrupt_pending_drawing_finished: bool,

    reg_interrupt_enable_drawing_started: bool,
    reg_interrupt_enable_start_of_frame_processing: bool,
    reg_interrupt_enable_drawing_finished: bool,

    reg_display_control_display_enable: bool,
    reg_display_control_sync_enable: bool,

    reg_drawing_control_drawing_enable: bool,

    reg_game_frame_control: usize,

    reg_bg_palette_0: u8,
    reg_bg_palette_1: u8,
    reg_bg_palette_2: u8,
    reg_bg_palette_3: u8,
    reg_obj_palette_0: u8,
    reg_obj_palette_1: u8,
    reg_obj_palette_2: u8,
    reg_obj_palette_3: u8,

    frame_clock_counter: u64,
    game_frame_clock_counter: usize,

    drawing_counter: u64,
}

impl Vip {
    pub fn new() -> Vip {
        Vip {
            vram: vec![0xff; VRAM_LENGTH as usize].into_boxed_slice(),

            display_state: DisplayState::Idle,

            drawing_state: DrawingState::Idle,

            reg_interrupt_pending_drawing_started: false,
            reg_interrupt_pending_start_of_frame_processing: false,
            reg_interrupt_pending_drawing_finished: false,

            reg_interrupt_enable_drawing_started: false,
            reg_interrupt_enable_start_of_frame_processing: false,
            reg_interrupt_enable_drawing_finished: false,

            reg_display_control_display_enable: false,
            reg_display_control_sync_enable: false,

            reg_drawing_control_drawing_enable: false,

            reg_game_frame_control: 1,

            reg_bg_palette_0: 0,
            reg_bg_palette_1: 0,
            reg_bg_palette_2: 0,
            reg_bg_palette_3: 0,
            reg_obj_palette_0: 0,
            reg_obj_palette_1: 0,
            reg_obj_palette_2: 0,
            reg_obj_palette_3: 0,

            frame_clock_counter: 0,
            game_frame_clock_counter: 0,

            drawing_counter: 0,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        match map_address(addr) {
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Attempted read byte from Interrupt Pending Reg");
                0
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Attempted read byte from Interrupt Enable Reg");
                0
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Attempted read byte from Interrupt Clear Reg");
                0
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted read byte from Display Control Read Reg");
                0
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted read byte from Display Control Write Reg");
                0
            }
            MappedAddress::LedBrightness1Reg => {
                println!("WARNING: Attempted read byte from LED Brightness 1 Reg");
                0
            }
            MappedAddress::LedBrightness2Reg => {
                println!("WARNING: Attempted read byte from LED Brightness 2 Reg");
                0
            }
            MappedAddress::LedBrightness3Reg => {
                println!("WARNING: Attempted read byte from LED Brightness 3 Reg");
                0
            }
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Attempted read byte from LED Brightness Idle Reg");
                0
            }
            MappedAddress::GameFrameControlReg => {
                println!("WARNING: Attempted read byte from Game Frame Control Reg");
                0
            }
            MappedAddress::DrawingControlReadReg => {
                println!("WARNING: Attempted read byte from Drawing Control Read Reg");
                0
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Attempted read byte from Drawing Control Write Reg");
                0
            }
            MappedAddress::ObjGroup0PointerReg => {
                println!("WARNING: Attempted read byte from OBJ Group 0 Pointer Reg");
                0
            }
            MappedAddress::ObjGroup1PointerReg => {
                println!("WARNING: Attempted read byte from OBJ Group 1 Pointer Reg");
                0
            }
            MappedAddress::ObjGroup2PointerReg => {
                println!("WARNING: Attempted read byte from OBJ Group 2 Pointer Reg");
                0
            }
            MappedAddress::ObjGroup3PointerReg => {
                println!("WARNING: Attempted read byte from OBJ Group 3 Pointer Reg");
                0
            }
            MappedAddress::BgPalette0Reg => self.reg_bg_palette_0,
            MappedAddress::BgPalette1Reg => self.reg_bg_palette_1,
            MappedAddress::BgPalette2Reg => self.reg_bg_palette_2,
            MappedAddress::BgPalette3Reg => self.reg_bg_palette_3,
            MappedAddress::ObjPalette0Reg => self.reg_obj_palette_0,
            MappedAddress::ObjPalette1Reg => self.reg_obj_palette_1,
            MappedAddress::ObjPalette2Reg => self.reg_obj_palette_2,
            MappedAddress::ObjPalette3Reg => self.reg_obj_palette_3,
            MappedAddress::ClearColorReg => {
                println!("WARNING: Attempted read byte from Clear Color Reg");
                0
            }
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize]
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted read byte from unrecognized VIP address (addr: 0x{:08x})", addr);
                0
            }
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match map_address(addr) {
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Attempted write byte to Interrupt Pending Reg (value: 0x{:02x})", value);
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Attempted write byte to Interrupt Enable Reg (value: 0x{:02x})", value);
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Attempted write byte to Interrupt Clear Reg (value: 0x{:02x})", value);
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted write byte to Display Control Read Reg (value: 0x{:02x})", value);
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted write byte to Display Control Write Reg (value: 0x{:02x})", value);
            }
            MappedAddress::LedBrightness1Reg => {
                println!("WARNING: Attempted write byte to LED Brightness 1 Reg (value: 0x{:02x})", value);
            }
            MappedAddress::LedBrightness2Reg => {
                println!("WARNING: Attempted write byte to LED Brightness 2 Reg (value: 0x{:02x})", value);
            }
            MappedAddress::LedBrightness3Reg => {
                println!("WARNING: Attempted write byte to LED Brightness 3 Reg (value: 0x{:02x})", value);
            }
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Attempted write byte to LED Brightness Idle Reg (value: 0x{:02x})", value);
            }
            MappedAddress::GameFrameControlReg => {
                println!("WARNING: Attempted write byte to Game Frame Control Reg (value: 0x{:02x})", value);
            }
            MappedAddress::DrawingControlReadReg => {
                println!("WARNING: Attempted write byte to Drawing Control Read Reg (value: 0x{:02x})", value);
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Attempted write byte to Drawing Control Write Reg (value: 0x{:02x})", value);
            }
            MappedAddress::ObjGroup0PointerReg => {
                println!("WARNING: Attempted write byte to OBJ Group 0 Pointer Reg (value: 0x{:02x})", value);
            }
            MappedAddress::ObjGroup1PointerReg => {
                println!("WARNING: Attempted write byte to OBJ Group 1 Pointer Reg (value: 0x{:02x})", value);
            }
            MappedAddress::ObjGroup2PointerReg => {
                println!("WARNING: Attempted write byte to OBJ Group 2 Pointer Reg (value: 0x{:02x})", value);
            }
            MappedAddress::ObjGroup3PointerReg => {
                println!("WARNING: Attempted write byte to OBJ Group 3 Pointer Reg (value: 0x{:02x})", value);
            }
            MappedAddress::BgPalette0Reg => self.reg_bg_palette_0 = value,
            MappedAddress::BgPalette1Reg => self.reg_bg_palette_1 = value,
            MappedAddress::BgPalette2Reg => self.reg_bg_palette_2 = value,
            MappedAddress::BgPalette3Reg => self.reg_bg_palette_3 = value,
            MappedAddress::ObjPalette0Reg => self.reg_obj_palette_0 = value,
            MappedAddress::ObjPalette1Reg => self.reg_obj_palette_1 = value,
            MappedAddress::ObjPalette2Reg => self.reg_obj_palette_2 = value,
            MappedAddress::ObjPalette3Reg => self.reg_obj_palette_3 = value,
            MappedAddress::ClearColorReg => {
                println!("WARNING: Attempted write byte to Clear Color Reg (value: 0x{:02x})", value);
            }
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value;
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted write byte to unrecognized VIP address (addr: 0x{:08x}, value: 0x{:02x})", addr, value);
            }
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Read halfword from Interrupt Pending Reg not fully implemented");
                (if self.reg_interrupt_pending_drawing_started { 1 } else { 0 } << 3) |
                (if self.reg_interrupt_pending_start_of_frame_processing { 1 } else { 0 } << 4) |
                (if self.reg_interrupt_pending_drawing_finished { 1 } else { 0 } << 14)
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Read halfword from Interrupt Enable Reg not fully implemented");
                (if self.reg_interrupt_enable_drawing_started { 1 } else { 0 } << 3) |
                (if self.reg_interrupt_enable_start_of_frame_processing { 1 } else { 0 } << 4) |
                (if self.reg_interrupt_enable_drawing_finished { 1 } else { 0 } << 14)
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Attempted read halfword from Interrupt Clear Reg");
                0
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Read halfword from Display Control Read Reg not fully implemented");
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
                println!("WARNING: Read halfword from Drawing Control Read Reg not fully implemented");
                match self.drawing_state {
                    DrawingState::Idle => 0,
                    DrawingState::Drawing => 0x000c,
                }
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
            MappedAddress::BgPalette0Reg => self.reg_bg_palette_0 as _,
            MappedAddress::BgPalette1Reg => self.reg_bg_palette_1 as _,
            MappedAddress::BgPalette2Reg => self.reg_bg_palette_2 as _,
            MappedAddress::BgPalette3Reg => self.reg_bg_palette_3 as _,
            MappedAddress::ObjPalette0Reg => self.reg_obj_palette_0 as _,
            MappedAddress::ObjPalette1Reg => self.reg_obj_palette_1 as _,
            MappedAddress::ObjPalette2Reg => self.reg_obj_palette_2 as _,
            MappedAddress::ObjPalette3Reg => self.reg_obj_palette_3 as _,
            MappedAddress::ClearColorReg => {
                println!("WARNING: Read halfword from Clear Color Reg not yet implemented");
                0
            }
            MappedAddress::Vram(addr) => {
                (self.vram[addr as usize] as u16) |
                ((self.vram[addr as usize + 1] as u16) << 8)
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted read halfword from unrecognized VIP address (addr: 0x{:08x})", addr);
                0
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
                println!("WARNING: Write halfword to Interrupt Enable Reg not fully implemented (value: 0x{:04x})", value);
                self.reg_interrupt_enable_drawing_started = (value & 0x0008) != 0;
                self.reg_interrupt_enable_start_of_frame_processing = (value & 0x0010) != 0;
                self.reg_interrupt_enable_drawing_finished = (value & 0x4000) != 0;
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Write halfword to Interrupt Clear Reg not fully implemented (value: 0x{:04x})", value);
                if (value & 0x0008) != 0 {
                    self.reg_interrupt_pending_drawing_started = false;
                }
                if (value & 0x0010) != 0 {
                    self.reg_interrupt_pending_start_of_frame_processing = false;
                }
                if (value & 0x4000) != 0 {
                    self.reg_interrupt_pending_drawing_finished = false;
                }
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted write halfword to Display Control Read Reg");
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Write halfword to Display Control Write Reg not fully implemented (value: 0x{:04x})", value);
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
                println!("WARNING: Write halfword to Drawing Control Write Reg not fully implemented (value: 0x{:04x})", value);
                self.reg_drawing_control_drawing_enable = (value & 0x02) != 0;
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
            MappedAddress::BgPalette0Reg => self.reg_bg_palette_0 = value as _,
            MappedAddress::BgPalette1Reg => self.reg_bg_palette_1 = value as _,
            MappedAddress::BgPalette2Reg => self.reg_bg_palette_2 = value as _,
            MappedAddress::BgPalette3Reg => self.reg_bg_palette_3 = value as _,
            MappedAddress::ObjPalette0Reg => self.reg_obj_palette_0 = value as _,
            MappedAddress::ObjPalette1Reg => self.reg_obj_palette_1 = value as _,
            MappedAddress::ObjPalette2Reg => self.reg_obj_palette_2 = value as _,
            MappedAddress::ObjPalette3Reg => self.reg_obj_palette_3 = value as _,
            MappedAddress::ClearColorReg => {
                println!("WARNING: Write halfword to Clear Color Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value as u8;
                self.vram[addr as usize + 1] = (value >> 8) as u8;
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted write halfword to unrecognized VIP address (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
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
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted read word from unrecognized VIP address (addr: 0x{:08x})", addr);
                0
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
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted write word to unrecognized VIP address (addr: 0x{:08x}, value: 0x{:08x})", addr, value);
            }
        }
    }

    fn read_vram_halfword(&self, addr: u32) -> u16 {
        (self.vram[addr as usize] as u16) |
        ((self.vram[addr as usize + 1] as u16) << 8)
    }

    pub fn cycles(&mut self, cycles: usize, video_driver: &mut VideoDriver) -> bool {
        let mut raise_interrupt = false;

        for _ in 0..cycles {
            self.frame_clock_counter += CPU_CYCLE_PERIOD_NS;
            if self.frame_clock_counter >= FRAME_CLOCK_PERIOD_NS {
                self.frame_clock_counter -= FRAME_CLOCK_PERIOD_NS;
                self.frame_clock(&mut raise_interrupt);
            }

            if let DrawingState::Drawing = self.drawing_state {
                self.drawing_counter += CPU_CYCLE_PERIOD_NS;
                if self.drawing_counter >= DRAWING_PERIOD_NS {
                    self.end_drawing_process(video_driver);
                    self.reg_interrupt_pending_drawing_finished = true;
                    if self.reg_interrupt_enable_drawing_finished {
                        raise_interrupt = true;
                    }
                }
            }
        }

        raise_interrupt
    }

    fn frame_clock(&mut self, raise_interrupt: &mut bool) {
        println!("Frame clock rising edge");

        self.reg_interrupt_pending_start_of_frame_processing = true;
        if self.reg_interrupt_enable_start_of_frame_processing {
            *raise_interrupt = true;
        }

        self.game_frame_clock_counter += 1;
        if self.game_frame_clock_counter >= self.reg_game_frame_control {
            self.game_frame_clock_counter = 0;
            self.game_clock(raise_interrupt);
        }
    }

    fn game_clock(&mut self, raise_interrupt: &mut bool) {
        println!("Game clock rising edge");

        if self.reg_drawing_control_drawing_enable {
            self.begin_drawing_process();
            self.reg_interrupt_pending_drawing_started = true;
            if self.reg_interrupt_enable_drawing_started {
                *raise_interrupt = true;
            }
        }
    }

    fn begin_drawing_process(&mut self) {
        println!("Begin drawing process");
        self.drawing_state = DrawingState::Drawing;
        self.drawing_counter = 0;
    }

    fn end_drawing_process(&mut self, video_driver: &mut VideoDriver) {
        const RESOLUTION_X: usize = 384;
        const RESOLUTION_Y: usize = 224;

        let mut buffer = vec![0; RESOLUTION_X * RESOLUTION_Y];

        const WINDOW_ENTRY_LENGTH: u32 = 32;
        let mut window_offset = WINDOW_ATTRIBS_END + 1 - WINDOW_ENTRY_LENGTH;
        let mut window_index = 31;
        for _ in 0..32 {
            println!("Window {}", window_index);

            let header = self.read_vram_halfword(window_offset);
            let base = (header & 0x000f) as u32;
            let stop = (header & 0x0040) != 0;
            let out_of_bounds = (header & 0x0080) != 0;
            let bg_height = ((header >> 8) & 0x03) as usize;
            let bg_width = ((header >> 10) & 0x03) as usize;
            let mode = ((header >> 12) & 0x03) as usize;
            let right_on = (header & 0x4000) != 0;
            let left_on = (header & 0x8000) != 0;
            println!(" Header: 0x{:04x}", header);
            println!("  base: 0x{:02x}", base);
            println!("  stop: {}", stop);
            println!("  out of bounds: {}", out_of_bounds);
            println!("  w, h: {}, {}", bg_width, bg_height);
            println!("  mode: {}", mode);
            println!("  l, r: {}, {}", left_on, right_on);

            let x = self.read_vram_halfword(window_offset + 2) as i16;
            let parallax = self.read_vram_halfword(window_offset + 4) as i16;
            let y = self.read_vram_halfword(window_offset + 6) as i16;
            let bg_x = self.read_vram_halfword(window_offset + 8) as i16;
            let bg_parallax = self.read_vram_halfword(window_offset + 10) as i16;
            let bg_y = self.read_vram_halfword(window_offset + 12) as i16;
            let width = self.read_vram_halfword(window_offset + 14);
            let height = self.read_vram_halfword(window_offset + 16);
            let param_base = self.read_vram_halfword(window_offset + 18) & 0xfff0;
            let out_of_bounds_char = self.read_vram_halfword(window_offset + 20);
            println!(" X: {}", x);
            println!(" Parallax: {}", parallax);
            println!(" Y: {}", y);
            println!(" BG X: {}", bg_x);
            println!(" BG Parallax: {}", bg_parallax);
            println!(" BG Y: {}", bg_y);
            println!(" Width: {}", width);
            println!(" Height: {}", height);
            println!(" Param base: 0x{:04x}", param_base);
            println!(" Out of bounds char: 0x{:04x}", out_of_bounds_char);

            if stop {
                break;
            }

            if left_on || right_on {
                let width = (width as u32) + 1;
                let height = (height as u32) + 1;
                let segment_offset = 0x00020000 + base * 0x00002000;

                for pixel_y in 0..RESOLUTION_Y as u32 {
                    for pixel_x in 0..RESOLUTION_X as u32 {
                        let window_x = pixel_x.wrapping_sub(x as u32);
                        let window_y = pixel_y.wrapping_sub(y as u32);

                        if window_x >= width || window_y >= height {
                            continue;
                        }

                        let background_x = window_x.wrapping_add(bg_x as u32);
                        let background_y = window_y.wrapping_add(bg_y as u32);

                        let segment_x = (background_x >> 3) & 0x3f;
                        let segment_y = (background_y >> 3) & 0x3f;
                        let mut offset_x = background_x & 0x07;
                        let mut offset_y = background_y & 0x07;
                        let segment_addr = segment_offset + (segment_y * 64 + segment_x) * 2;
                        let entry = self.read_vram_halfword(segment_addr as _);
                        let pal = (entry >> 14) & 0x03;
                        let horizontal_flip = (entry & 0x2000) != 0;
                        let vertical_flip = (entry & 0x1000) != 0;
                        if horizontal_flip {
                            offset_x = 7 - offset_x;
                        }
                        if vertical_flip {
                            offset_y = 7 - offset_y;
                        }
                        let char_index = (entry & 0x07ff) as u32;

                        let char_offset = if char_index < 0x0200 {
                            0x00006000 + char_index * 16
                        } else if char_index < 0x0400 {
                            0x0000e000 + (char_index - 0x0200) * 16
                        } else if char_index < 0x0600 {
                            0x00016000 + (char_index - 0x0400) * 16
                        } else {
                            0x0001e000 + (char_index - 0x0600) * 16
                        };

                        let char_row_offset = char_offset + offset_y * 2;
                        let char_row_data = self.read_vram_halfword(char_row_offset as _);
                        let palette_index = ((char_row_data as u32) >> (offset_x * 2)) & 0x03;

                        if palette_index == 0 {
                            continue;
                        }

                        let palette = match pal {
                            0 => self.reg_bg_palette_0,
                            1 => self.reg_bg_palette_1,
                            2 => self.reg_bg_palette_2,
                            _ => self.reg_bg_palette_3
                        };

                        let color = (((palette >> (palette_index * 2)) & 0x03) as u32) << 6;
                        buffer[(pixel_y as usize) * RESOLUTION_X + (pixel_x as usize)] = color << 16;
                    }
                }
            }

            window_offset -= WINDOW_ENTRY_LENGTH;
            window_index -= 1;
        }

        video_driver.output_frame(&buffer);

        println!("End drawing process");
        self.drawing_state = DrawingState::Idle;
    }
}
