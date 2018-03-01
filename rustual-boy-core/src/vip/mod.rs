mod mem_map;

use sinks::*;

use self::mem_map::*;

use std::cmp::*;

const FRAMEBUFFER_RESOLUTION_X: u32 = 384;
const FRAMEBUFFER_RESOLUTION_Y: u32 = 256;

pub const DISPLAY_RESOLUTION_X: u32 = 384;
pub const DISPLAY_RESOLUTION_Y: u32 = 224;
pub const DISPLAY_PIXELS: u32 = DISPLAY_RESOLUTION_X * DISPLAY_RESOLUTION_Y;

const DRAWING_BLOCK_HEIGHT: u32 = 8;
const DRAWING_BLOCK_COUNT: u32 = DISPLAY_RESOLUTION_Y / DRAWING_BLOCK_HEIGHT;

// 20mhz / (1s / 2.5ms) = 50000 clocks
const DISPLAY_FRAME_EIGHTH_PERIOD: u32 = 50000;

const DRAWING_PERIOD: u32 = DISPLAY_FRAME_EIGHTH_PERIOD * 2;
const DRAWING_BLOCK_PERIOD: u32 = DRAWING_PERIOD / DRAWING_BLOCK_COUNT;

// 20mhz / (1s / 56us) = 1120 clocks
const DRAWING_SBOUT_PERIOD: u32 = 1120;

enum DisplayState {
    Idle,
    LeftFramebuffer,
    RightFramebuffer,
    Finished,
}

#[derive(Eq, PartialEq)]
enum DrawingState {
    Idle,
    Drawing,
}

enum Eye {
    Left,
    Right,
}

#[derive(Eq, PartialEq)]
enum WindowMode {
    Normal,
    LineShift,
    Affine,
    Obj,
}

#[derive(Clone, Copy, Debug)]
enum ObjGroup {
    Group0,
    Group1,
    Group2,
    Group3,
}

pub struct Vip {
    _vram: Box<[u8]>,
    vram_ptr: *mut u8,

    display_state: DisplayState,

    drawing_state: DrawingState,

    reg_intpnd_lfbend: bool,
    reg_intpnd_rfbend: bool,
    reg_intpnd_gamestart: bool,
    reg_intpnd_framestart: bool,
    reg_intpnd_sbhit: bool,
    reg_intpnd_xpend: bool,

    reg_intenb_lfbend: bool,
    reg_intenb_rfbend: bool,
    reg_intenb_gamestart: bool,
    reg_intenb_framestart: bool,
    reg_intenb_sbhit: bool,
    reg_intenb_xpend: bool,

    reg_dpctrl_disp: bool,
    reg_dpctrl_synce: bool,

    reg_xpctrl_xpen: bool,
    reg_xpctrl_sbcount: u32,
    reg_xpctrl_sbcmp: u32,
    reg_xpctrl_sbout: bool,

    reg_frmcyc: u32,

    reg_brta: u8,
    reg_brtb: u8,
    reg_brtc: u8,

    reg_spt0: u16,
    reg_spt1: u16,
    reg_spt2: u16,
    reg_spt3: u16,

    reg_gplt0: u8,
    reg_gplt1: u8,
    reg_gplt2: u8,
    reg_gplt3: u8,

    reg_jplt0: u8,
    reg_jplt1: u8,
    reg_jplt2: u8,
    reg_jplt3: u8,

    reg_bkcol: u8,

    display_frame_eighth_clock_counter: u32,
    display_frame_eighth_counter: u32,

    drawing_block_counter: u32,
    drawing_sbout_counter: u32,

    fclk: u32,

    display_first_framebuffers: bool,
    last_bkcol: u8,

    gamma_table: Box<[u8; 256]>,
}

impl Vip {
    pub fn new() -> Vip {
        let mut vram = vec![0; VRAM_LENGTH as usize].into_boxed_slice();
        let vram_ptr = vram.as_mut_ptr();

        let gamma = 1.0 / 2.2;
        let mut gamma_table = Box::new([0; 256]);
        for (i, entry) in gamma_table.iter_mut().enumerate() {
            *entry = min(max((((i as f64) / 255.0).powf(gamma) * 255.0) as i32, 0), 255) as u8;
        }

        Vip {
            _vram: vram,
            vram_ptr: vram_ptr,

            display_state: DisplayState::Idle,

            drawing_state: DrawingState::Idle,

            reg_intpnd_lfbend: false,
            reg_intpnd_rfbend: false,
            reg_intpnd_gamestart: false,
            reg_intpnd_framestart: false,
            reg_intpnd_sbhit: false,
            reg_intpnd_xpend: false,

            reg_intenb_lfbend: false,
            reg_intenb_rfbend: false,
            reg_intenb_gamestart: false,
            reg_intenb_framestart: false,
            reg_intenb_sbhit: false,
            reg_intenb_xpend: false,

            reg_dpctrl_disp: false,
            reg_dpctrl_synce: false,

            reg_xpctrl_xpen: false,
            reg_xpctrl_sbcount: 0,
            reg_xpctrl_sbcmp: 0,
            reg_xpctrl_sbout: false,

            reg_frmcyc: 0,

            reg_brta: 0,
            reg_brtb: 0,
            reg_brtc: 0,

            reg_spt0: 0,
            reg_spt1: 0,
            reg_spt2: 0,
            reg_spt3: 0,

            reg_gplt0: 0,
            reg_gplt1: 0,
            reg_gplt2: 0,
            reg_gplt3: 0,

            reg_jplt0: 0,
            reg_jplt1: 0,
            reg_jplt2: 0,
            reg_jplt3: 0,

            reg_bkcol: 0,

            display_frame_eighth_clock_counter: 0,
            display_frame_eighth_counter: 0,

            drawing_block_counter: 0,
            drawing_sbout_counter: 0,

            fclk: 0,

            display_first_framebuffers: false,
            last_bkcol: 0,

            gamma_table: gamma_table,
        }
    }

    fn reg_intpnd(&self) -> u16 {
        (if self.reg_intpnd_lfbend { 1 } else { 0 } << 1) |
        (if self.reg_intpnd_rfbend { 1 } else { 0 } << 2) |
        (if self.reg_intpnd_gamestart { 1 } else { 0 } << 3) |
        (if self.reg_intpnd_framestart { 1 } else { 0 } << 4) |
        (if self.reg_intpnd_sbhit { 1 } else { 0 } << 13) |
        (if self.reg_intpnd_xpend { 1 } else { 0 } << 14)
    }

    fn reg_intenb(&self) -> u16 {
        (if self.reg_intenb_lfbend { 1 } else { 0 } << 1) |
        (if self.reg_intenb_rfbend { 1 } else { 0 } << 2) |
        (if self.reg_intenb_gamestart { 1 } else { 0 } << 3) |
        (if self.reg_intenb_framestart { 1 } else { 0 } << 4) |
        (if self.reg_intenb_sbhit { 1 } else { 0 } << 13) |
        (if self.reg_intenb_xpend { 1 } else { 0 } << 14)
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let addr = addr & 0x0007ffff;
        match addr {
            VRAM_START ... VRAM_END => self.read_vram_byte(addr - VRAM_START),
            CHR_RAM_PATTERN_TABLE_0_MIRROR_START ... CHR_RAM_PATTERN_TABLE_0_MIRROR_END =>
                self.read_vram_byte(addr - CHR_RAM_PATTERN_TABLE_0_MIRROR_START + CHR_RAM_PATTERN_TABLE_0_START),
            CHR_RAM_PATTERN_TABLE_1_MIRROR_START ... CHR_RAM_PATTERN_TABLE_1_MIRROR_END =>
                self.read_vram_byte(addr - CHR_RAM_PATTERN_TABLE_1_MIRROR_START + CHR_RAM_PATTERN_TABLE_1_START),
            CHR_RAM_PATTERN_TABLE_2_MIRROR_START ... CHR_RAM_PATTERN_TABLE_2_MIRROR_END =>
                self.read_vram_byte(addr - CHR_RAM_PATTERN_TABLE_2_MIRROR_START + CHR_RAM_PATTERN_TABLE_2_START),
            CHR_RAM_PATTERN_TABLE_3_MIRROR_START ... CHR_RAM_PATTERN_TABLE_3_MIRROR_END =>
                self.read_vram_byte(addr - CHR_RAM_PATTERN_TABLE_3_MIRROR_START + CHR_RAM_PATTERN_TABLE_3_START),
            _ => {
                let halfword = self.read_halfword(addr & 0xfffffffe);
                if (addr & 0x01) == 0 {
                    halfword as _
                } else {
                    (halfword >> 8) as _
                }
            }
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = addr & 0x0007ffff;
        match addr {
            VRAM_START ... VRAM_END => self.write_vram_byte(addr - VRAM_START, value),
            CHR_RAM_PATTERN_TABLE_0_MIRROR_START ... CHR_RAM_PATTERN_TABLE_0_MIRROR_END =>
                self.write_vram_byte(addr - CHR_RAM_PATTERN_TABLE_0_MIRROR_START + CHR_RAM_PATTERN_TABLE_0_START, value),
            CHR_RAM_PATTERN_TABLE_1_MIRROR_START ... CHR_RAM_PATTERN_TABLE_1_MIRROR_END =>
                self.write_vram_byte(addr - CHR_RAM_PATTERN_TABLE_1_MIRROR_START + CHR_RAM_PATTERN_TABLE_1_START, value),
            CHR_RAM_PATTERN_TABLE_2_MIRROR_START ... CHR_RAM_PATTERN_TABLE_2_MIRROR_END =>
                self.write_vram_byte(addr - CHR_RAM_PATTERN_TABLE_2_MIRROR_START + CHR_RAM_PATTERN_TABLE_2_START, value),
            CHR_RAM_PATTERN_TABLE_3_MIRROR_START ... CHR_RAM_PATTERN_TABLE_3_MIRROR_END =>
                self.write_vram_byte(addr - CHR_RAM_PATTERN_TABLE_3_MIRROR_START + CHR_RAM_PATTERN_TABLE_3_START, value),
            _ => {
                let halfword = if (addr & 0x01) == 0 {
                    value as _
                } else {
                    (value as u16) << 8
                };
                self.write_halfword(addr & 0xfffffffe, halfword);
            }
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0x0007ffff;
        let addr = addr & 0xfffffffe;
        match addr {
            VRAM_START ... VRAM_END => self.read_vram_halfword(addr - VRAM_START),
            INTPND => {
                logln!(Log::Vip, "WARNING: Read halfword from INTPND not fully implemented");
                self.reg_intpnd()
            }
            INTENB => {
                logln!(Log::Vip, "WARNING: Read halfword from INTENB not fully implemented");
                self.reg_intenb()
            }
            INTCLR => {
                logln!(Log::Vip, "WARNING: Attempted read halfword from INTCLR");
                0
            }
            DPSTTS => {
                let scanrdy = true; // TODO
                let re = true; // TODO
                let column_table_addr_lock = false; // TODO

                (if self.reg_dpctrl_disp { 1 } else { 0 } << 1) |
                (match self.display_state {
                    DisplayState::Idle | DisplayState::Finished => 0b0000,
                    DisplayState::LeftFramebuffer => if self.display_first_framebuffers { 0b0001 } else { 0b0100 },
                    DisplayState::RightFramebuffer => if self.display_first_framebuffers { 0b0010 } else { 0b1000 },
                } << 2) |
                (if scanrdy { 1 } else { 0 } << 6) |
                (if self.display_frame_eighth_counter < 4 { 1 } else { 0 } << 7) |
                (if re { 1 } else { 0 } << 8) |
                (if self.reg_dpctrl_synce { 1 } else { 0 } << 9) |
                (if column_table_addr_lock { 1 } else { 0 } << 10)
            }
            DPCTRL => {
                logln!(Log::Vip, "WARNING: Attempted read halfword from DPCTRL");
                0
            }
            BRTA => self.reg_brta as _,
            BRTB => self.reg_brtb as _,
            BRTC => self.reg_brtc as _,
            REST => {
                logln!(Log::Vip, "WARNING: Read halfword from REST not yet implemented");
                0
            }
            FRMCYC => {
                self.reg_frmcyc as u16
            }
            XPSTTS => {
                let draw_to_first_framebuffers = !self.display_first_framebuffers;
                let (drawing_to_frame_buffer_0, drawing_to_frame_buffer_1) = match self.drawing_state {
                    DrawingState::Drawing => {
                        if draw_to_first_framebuffers {
                            (true, false)
                        } else {
                            (false, true)
                        }
                    }
                    _ => (false, false)
                };
                let drawing_exceeds_frame_period = false;

                (if self.reg_xpctrl_xpen { 1 } else { 0 } << 1) |
                (if drawing_to_frame_buffer_0 { 1 } else { 0 } << 2) |
                (if drawing_to_frame_buffer_1 { 1 } else { 0 } << 3) |
                (if drawing_exceeds_frame_period { 1 } else { 0 } << 4) |
                ((self.reg_xpctrl_sbcount as u16) << 8) |
                // TODO: This particular bit seems to strobe much faster than we do here on hw, look more into that
                (if self.reg_xpctrl_sbout { 1 } else { 0 } << 15)
            }
            XPCTRL => {
                logln!(Log::Vip, "WARNING: Attempted read halfword from XPCTRL");
                0
            }
            SPT0 => self.reg_spt0,
            SPT1 => self.reg_spt1,
            SPT2 => self.reg_spt2,
            SPT3 => self.reg_spt3,
            GPLT0 => self.reg_gplt0 as _,
            GPLT1 => self.reg_gplt1 as _,
            GPLT2 => self.reg_gplt2 as _,
            GPLT3 => self.reg_gplt3 as _,
            JPLT0 => self.reg_jplt0 as _,
            JPLT1 => self.reg_jplt1 as _,
            JPLT2 => self.reg_jplt2 as _,
            JPLT3 => self.reg_jplt3 as _,
            BKCOL => self.reg_bkcol as _,
            CHR_RAM_PATTERN_TABLE_0_MIRROR_START ... CHR_RAM_PATTERN_TABLE_0_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_0_MIRROR_START + CHR_RAM_PATTERN_TABLE_0_START),
            CHR_RAM_PATTERN_TABLE_1_MIRROR_START ... CHR_RAM_PATTERN_TABLE_1_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_1_MIRROR_START + CHR_RAM_PATTERN_TABLE_1_START),
            CHR_RAM_PATTERN_TABLE_2_MIRROR_START ... CHR_RAM_PATTERN_TABLE_2_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_2_MIRROR_START + CHR_RAM_PATTERN_TABLE_2_START),
            CHR_RAM_PATTERN_TABLE_3_MIRROR_START ... CHR_RAM_PATTERN_TABLE_3_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_3_MIRROR_START + CHR_RAM_PATTERN_TABLE_3_START),
            _ => {
                logln!(Log::Vip, "WARNING: Attempted read halfword from unrecognized VIP address (addr: 0x{:08x})", addr);
                0
            }
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0x0007ffff;
        let addr = addr & 0xfffffffe;
        match addr {
            VRAM_START ... VRAM_END => self.write_vram_halfword(addr - VRAM_START, value),
            INTPND => {
                logln!(Log::Vip, "WARNING: Attempted write halfword to Interrupt Pending Reg");
            }
            INTENB => {
                logln!(Log::Vip, "WARNING: Write halfword to INTENB not fully implemented (value: 0x{:04x})", value);
                self.reg_intenb_lfbend = (value & 0x0002) != 0;
                self.reg_intenb_rfbend = (value & 0x0004) != 0;
                self.reg_intenb_gamestart = (value & 0x0008) != 0;
                self.reg_intenb_framestart = (value & 0x0010) != 0;
                self.reg_intenb_sbhit = (value & 0x2000) != 0;
                self.reg_intenb_xpend = (value & 0x4000) != 0;
            }
            INTCLR => {
                logln!(Log::Vip, "WARNING: Write halfword to INTCLR not fully implemented (value: 0x{:04x})", value);
                if (value & 0x0002) != 0 {
                    self.reg_intpnd_lfbend = false;
                }
                if (value & 0x0004) != 0 {
                    self.reg_intpnd_rfbend = false;
                }
                if (value & 0x0008) != 0 {
                    self.reg_intpnd_gamestart = false;
                }
                if (value & 0x0010) != 0 {
                    self.reg_intpnd_framestart = false;
                }
                if (value & 0x2000) != 0 {
                    self.reg_intpnd_sbhit = false;
                }
                if (value & 0x4000) != 0 {
                    self.reg_intpnd_xpend = false;
                }
            }
            DPSTTS => {
                logln!(Log::Vip, "WARNING: Attempted write halfword to dpstts reg");
            }
            DPCTRL => {
                logln!(Log::Vip, "WARNING: Write halfword to DPCTRL not fully implemented (value: 0x{:04x})", value);

                let dprst = (value & 0x0001) != 0;
                let disp = (value & 0x0002) != 0;
                let _re = (value & 0x0100) != 0; // TODO
                self.reg_dpctrl_synce = (value & 0x0200) != 0;
                let _lock = (value & 0x0400) != 0;

                if dprst {
                    self.display_state = DisplayState::Finished;

                    self.reg_intpnd_gamestart = false;
                    self.reg_intpnd_framestart = false;
                    self.reg_intpnd_lfbend = false;
                    self.reg_intpnd_rfbend = false;
                    self.reg_intenb_gamestart = false;
                    self.reg_intenb_framestart = false;
                    self.reg_intenb_lfbend = false;
                    self.reg_intenb_rfbend = false;
                } else if disp && !self.reg_dpctrl_disp {
                    self.display_state = DisplayState::Finished;
                }

                self.reg_dpctrl_disp = disp;
            }
            BRTA => self.reg_brta = value as _,
            BRTB => self.reg_brtb = value as _,
            BRTC => self.reg_brtc = value as _,
            REST => {
                logln!(Log::Vip, "WARNING: Write halfword to REST not yet implemented (value: 0x{:04x})", value);
            }
            FRMCYC => {
                logln!(Log::Vip, "FRMCYC written (value: 0x{:04x})", value);
                self.reg_frmcyc = value as u32;
            }
            XPSTTS => {
                logln!(Log::Vip, "WARNING: Attempted write halfword to XPSTTS (value: 0x{:04x})", value);
            }
            XPCTRL => {
                logln!(Log::Vip, "WARNING: Write halfword to XPCTRL not fully implemented (value: 0x{:04x})", value);

                let xprst = (value & 0x01) != 0;
                self.reg_xpctrl_xpen = (value & 0x02) != 0;
                self.reg_xpctrl_sbcmp = ((value as u32) >> 8) & 0x1f;

                if xprst {
                    self.drawing_state = DrawingState::Idle;

                    self.reg_intpnd_xpend = false;
                    self.reg_intenb_xpend = false;
                }
            }
            SPT0 => self.reg_spt0 = value & 0x03ff,
            SPT1 => self.reg_spt1 = value & 0x03ff,
            SPT2 => self.reg_spt2 = value & 0x03ff,
            SPT3 => self.reg_spt3 = value & 0x03ff,
            GPLT0 => self.reg_gplt0 = value as _,
            GPLT1 => self.reg_gplt1 = value as _,
            GPLT2 => self.reg_gplt2 = value as _,
            GPLT3 => self.reg_gplt3 = value as _,
            JPLT0 => self.reg_jplt0 = value as _,
            JPLT1 => self.reg_jplt1 = value as _,
            JPLT2 => self.reg_jplt2 = value as _,
            JPLT3 => self.reg_jplt3 = value as _,
            BKCOL => self.reg_bkcol = (value & 0x03) as _,
            CHR_RAM_PATTERN_TABLE_0_MIRROR_START ... CHR_RAM_PATTERN_TABLE_0_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_0_MIRROR_START + CHR_RAM_PATTERN_TABLE_0_START, value),
            CHR_RAM_PATTERN_TABLE_1_MIRROR_START ... CHR_RAM_PATTERN_TABLE_1_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_1_MIRROR_START + CHR_RAM_PATTERN_TABLE_1_START, value),
            CHR_RAM_PATTERN_TABLE_2_MIRROR_START ... CHR_RAM_PATTERN_TABLE_2_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_2_MIRROR_START + CHR_RAM_PATTERN_TABLE_2_START, value),
            CHR_RAM_PATTERN_TABLE_3_MIRROR_START ... CHR_RAM_PATTERN_TABLE_3_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_3_MIRROR_START + CHR_RAM_PATTERN_TABLE_3_START, value),
            _ => {
                logln!(Log::Vip, "WARNING: Attempted write halfword to unrecognized VIP address (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
            }
        }
    }

    fn read_vram_byte(&self, addr: u32) -> u8 {
        unsafe {
            *self.vram_ptr.offset(addr as _)
        }
    }

    fn write_vram_byte(&mut self, addr: u32, value: u8) {
        unsafe {
            *self.vram_ptr.offset(addr as _) = value;
        }
    }

    fn read_vram_halfword(&self, addr: u32) -> u16 {
        unsafe {
            (*self.vram_ptr.offset(addr as _) as u16) |
            ((*self.vram_ptr.offset((addr + 1) as _) as u16) << 8)
        }
    }

    fn write_vram_halfword(&mut self, addr: u32, value: u16) {
        unsafe {
            *self.vram_ptr.offset(addr as _) = value as _;
            *self.vram_ptr.offset((addr + 1) as _) = (value >> 8) as _;
        }
    }

    pub fn cycles(&mut self, cycles: u32, video_frame_sink: &mut VideoSink) -> bool {
        for _ in 0..cycles {
            self.display_frame_eighth_clock_counter += 1;
            if self.display_frame_eighth_clock_counter >= DISPLAY_FRAME_EIGHTH_PERIOD {
                self.display_frame_eighth_clock_counter = 0;

                self.display_frame_eighth_counter = match self.display_frame_eighth_counter {
                    7 => 0,
                    _ => self.display_frame_eighth_counter + 1
                };

                match self.display_frame_eighth_counter {
                    0 => {
                        self.frame_clock();
                    }
                    1 => {
                        self.display(video_frame_sink);

                        if self.reg_dpctrl_disp && self.reg_dpctrl_synce {
                            self.begin_left_framebuffer_display_process();
                        }
                    }
                    3 => {
                        if self.reg_dpctrl_disp {
                            if let DisplayState::LeftFramebuffer = self.display_state {
                                self.end_left_framebuffer_display_process();
                            }
                        }
                    }
                    5 => {
                        if self.reg_dpctrl_disp && self.reg_dpctrl_synce {
                            self.begin_right_framebuffer_display_process();
                        }
                    }
                    7 => {
                        if self.reg_dpctrl_disp {
                            if let DisplayState::RightFramebuffer = self.display_state {
                                self.reg_intpnd_rfbend = true;
                            }

                            self.end_display_process();
                        }
                    }
                    _ => {}
                }
            }

            if let DrawingState::Drawing = self.drawing_state {
                self.drawing_block_counter += 1;
                if self.drawing_block_counter >= DRAWING_BLOCK_PERIOD {
                    self.drawing_block_counter = 0;

                    if self.reg_xpctrl_sbcount < DRAWING_BLOCK_COUNT {
                        self.end_drawing_block();

                        if self.reg_xpctrl_sbcount < DRAWING_BLOCK_COUNT - 1 {
                            self.reg_xpctrl_sbcount += 1;
                            if self.reg_xpctrl_xpen {
                                self.begin_drawing_block();
                            }
                        } else {
                            self.end_drawing_process();
                            self.reg_intpnd_xpend = true;
                        }
                    }
                }

                if self.reg_xpctrl_sbout {
                    self.drawing_sbout_counter += 1;
                    if self.drawing_sbout_counter >= DRAWING_SBOUT_PERIOD {
                        self.reg_xpctrl_sbout = false;
                    }
                }
            }
        }

        // Always raise any pending interrupts if the corresponding interrupts are enabled
        (self.reg_intpnd() & self.reg_intenb()) != 0
    }

    fn frame_clock(&mut self) {
        logln!(Log::Vip, "Frame clock rising edge");

        self.reg_intpnd_framestart = true;

        if self.reg_dpctrl_disp {
            self.begin_display_process();
        }

        self.fclk += 1;
        if self.fclk > self.reg_frmcyc {
            self.fclk = 0;
            self.game_clock();
        }
    }

    fn game_clock(&mut self) {
        logln!(Log::Vip, "Game clock rising edge");

        self.reg_intpnd_gamestart = true;

        if self.reg_xpctrl_xpen {
            self.display_first_framebuffers = !self.display_first_framebuffers;

            self.begin_drawing_process();
        } else {
            self.reg_intpnd_xpend = true;
        }
    }

    fn begin_drawing_process(&mut self) {
        logln!(Log::Vip, "Begin drawing process");
        self.drawing_state = DrawingState::Drawing;

        self.reg_xpctrl_sbcount = 0;

        self.drawing_block_counter = 0;

        self.begin_drawing_block();
    }

    fn begin_drawing_block(&mut self) {
        logln!(Log::Vip, "Begin drawing block {}", self.reg_xpctrl_sbcount);
    }

    fn end_drawing_block(&mut self) {
        logln!(Log::Vip, "End drawing block {}", self.reg_xpctrl_sbcount);

        self.draw_current_block();

        if self.reg_xpctrl_sbcount == self.reg_xpctrl_sbcmp {
            self.reg_xpctrl_sbout = true;
            self.drawing_sbout_counter = 0;

            self.reg_intpnd_sbhit = true;
        }
    }

    fn end_drawing_process(&mut self) {
        logln!(Log::Vip, "End drawing process");
        self.drawing_state = DrawingState::Idle;
    }

    fn begin_display_process(&mut self) {
        logln!(Log::Vip, "Start display process");
        self.display_state = DisplayState::Idle;
    }

    fn begin_left_framebuffer_display_process(&mut self) {
        logln!(Log::Vip, "Start left framebuffer display process");
        self.display_state = DisplayState::LeftFramebuffer;
    }

    fn end_left_framebuffer_display_process(&mut self) {
        logln!(Log::Vip, "End left framebuffer display process");

        self.display_state = DisplayState::Idle;

        self.reg_intpnd_lfbend = true;
    }

    fn begin_right_framebuffer_display_process(&mut self) {
        logln!(Log::Vip, "Start right framebuffer display process");
        self.display_state = DisplayState::RightFramebuffer;
    }

    fn end_display_process(&mut self) {
        logln!(Log::Vip, "End display process");
        self.display_state = DisplayState::Finished;
    }

    fn draw_current_block(&mut self) {
        let draw_to_first_framebuffers = !self.display_first_framebuffers;
        let left_framebuffer_offset = if draw_to_first_framebuffers { 0x00000000 } else { 0x00008000 };
        let right_framebuffer_offset = left_framebuffer_offset + 0x00010000;

        let block_start_y = self.reg_xpctrl_sbcount * 8;
        let block_end_y = block_start_y + DRAWING_BLOCK_HEIGHT;

        let clear_pixels = (self.last_bkcol << 6) | (self.last_bkcol << 4) | (self.last_bkcol << 2) | self.last_bkcol;
        for x in 0..FRAMEBUFFER_RESOLUTION_X {
            let column_offset = (x * FRAMEBUFFER_RESOLUTION_Y + block_start_y) / 4;
            self.write_vram_byte(left_framebuffer_offset + column_offset, clear_pixels);
            self.write_vram_byte(left_framebuffer_offset + column_offset + 1, clear_pixels);
            self.write_vram_byte(right_framebuffer_offset + column_offset, clear_pixels);
            self.write_vram_byte(right_framebuffer_offset + column_offset + 1, clear_pixels);
        }
        // Latch clear color reg _after_ each block. This is a known (and documented) hardware bug.
        self.last_bkcol = self.reg_bkcol;

        let mut current_obj_group = Some(ObjGroup::Group3);

        const WINDOW_ENTRY_LENGTH: u32 = 32;
        let mut window_offset = WINDOW_ATTRIBS_END + 1 - WINDOW_ENTRY_LENGTH;
        let mut window_index = 31;
        for _ in 0..32 {
            logln!(Log::Vip, "Window {}", window_index);

            let header = self.read_vram_halfword(window_offset);
            logln!(Log::Vip, " Header: 0x{:04x}", header);

            if header == 0 {
                logln!(Log::Vip, "  [Dummy world]");
            } else {
                let base = (header & 0x000f) as u32;
                let stop = (header & 0x0040) != 0;
                let overplane = (header & 0x0080) != 0;
                let bg_height = ((header >> 8) & 0x03) as u32;
                let bg_width = ((header >> 10) & 0x03) as u32;
                let mode = ((header >> 12) & 0x03) as u32;
                let right_on = (header & 0x4000) != 0;
                let left_on = (header & 0x8000) != 0;
                /*logln!(Log::Vip, "  base: 0x{:02x}", base);
                logln!(Log::Vip, "  stop: {}", stop);
                logln!(Log::Vip, "  overplane: {}", overplane);
                logln!(Log::Vip, "  w, h: {}, {}", bg_width, bg_height);
                logln!(Log::Vip, "  mode: {}", mode);
                logln!(Log::Vip, "  l, r: {}, {}", left_on, right_on);*/

                let x = self.read_vram_halfword(window_offset + 2) as i16;
                let parallax = self.read_vram_halfword(window_offset + 4) as i16;
                let y = self.read_vram_halfword(window_offset + 6) as i16;
                let bg_x = self.read_vram_halfword(window_offset + 8) as i16;
                let bg_parallax = self.read_vram_halfword(window_offset + 10) as i16;
                let bg_y = self.read_vram_halfword(window_offset + 12) as i16;
                let width = self.read_vram_halfword(window_offset + 14);
                let height = self.read_vram_halfword(window_offset + 16);
                let param_base = self.read_vram_halfword(window_offset + 18) as u32;
                let overplane_char = self.read_vram_halfword(window_offset + 20);
                /*logln!(Log::Vip, " X: {}", x);
                logln!(Log::Vip, " Parallax: {}", parallax);
                logln!(Log::Vip, " Y: {}", y);
                logln!(Log::Vip, " BG X: {}", bg_x);
                logln!(Log::Vip, " BG Parallax: {}", bg_parallax);
                logln!(Log::Vip, " BG Y: {}", bg_y);
                logln!(Log::Vip, " Width: {}", width);
                logln!(Log::Vip, " Height: {}", height);
                logln!(Log::Vip, " Param base: 0x{:04x}", param_base);
                logln!(Log::Vip, " Overplane char: 0x{:04x}", overplane_char);*/

                if stop {
                    break;
                }

                let width = (width as u32) + 1;
                let height = (height as u32) + 1;
                let segment_base = 0x00020000 + base * 0x00002000;
                let segments_x = 1 << bg_width;
                let segments_y = 1 << bg_height;
                let param_offset = 0x00020000 + param_base * 2;
                let overplane_char_entry = self.read_vram_halfword(0x00020000 + (overplane_char as u32) * 2);

                let mode = match mode {
                    0 => WindowMode::Normal,
                    1 => WindowMode::LineShift,
                    2 => WindowMode::Affine,
                    _ => WindowMode::Obj
                };

                for i in 0..2 {
                    let eye = match i {
                        0 => Eye::Left,
                        _ => Eye::Right,
                    };

                    match eye {
                        Eye::Left => {
                            if !left_on {
                                continue;
                            }
                        }
                        Eye::Right => {
                            if !right_on {
                                continue;
                            }
                        }
                    }

                    let framebuffer_offset = match eye {
                        Eye::Left => left_framebuffer_offset,
                        Eye::Right => right_framebuffer_offset,
                    };

                    match mode {
                        WindowMode::Obj => {
                            //logln!(Log::Vip, "Current obj group: {:?}", current_obj_group);

                            match current_obj_group {
                                Some(obj_group) => {
                                    let starting_obj_index = match obj_group {
                                        ObjGroup::Group0 => self.reg_spt0,
                                        ObjGroup::Group1 => self.reg_spt1,
                                        ObjGroup::Group2 => self.reg_spt2,
                                        ObjGroup::Group3 => self.reg_spt3,
                                    };
                                    let mut ending_obj_index = match obj_group {
                                        ObjGroup::Group0 => 0,
                                        ObjGroup::Group1 => self.reg_spt0 + 1,
                                        ObjGroup::Group2 => self.reg_spt1 + 1,
                                        ObjGroup::Group3 => self.reg_spt2 + 1,
                                    };
                                    if ending_obj_index >= starting_obj_index {
                                        ending_obj_index = 0;
                                    }
                                    for i in (ending_obj_index..starting_obj_index + 1).rev() {
                                        //logln!(Log::Vip, "Current obj: {}", i);

                                        let obj_offset = 0x0003e000 + (i as u32) * 8;

                                        let x = self.read_vram_halfword(obj_offset) as i16;
                                        let l_r_parallax = self.read_vram_halfword(obj_offset + 2);
                                        let l = (l_r_parallax & 0x8000) != 0;
                                        let r = (l_r_parallax & 0x4000) != 0;
                                        let parallax = ((l_r_parallax << 2) as i16) >> 2;
                                        let y = self.read_vram_halfword(obj_offset + 4) as i16;
                                        let pal_hf_vf_char = self.read_vram_halfword(obj_offset + 6);
                                        let pal = pal_hf_vf_char >> 14;
                                        let horizontal_flip = (pal_hf_vf_char & 0x2000) != 0;
                                        let vertical_flip = (pal_hf_vf_char & 0x1000) != 0;
                                        let char_index = (pal_hf_vf_char & 0x07ff) as u32;
                                        /*logln!(Log::Vip, " X: {}", x);
                                        logln!(Log::Vip, " L: {}", l);
                                        logln!(Log::Vip, " R: {}", r);
                                        logln!(Log::Vip, " Parallax: {}", parallax);
                                        logln!(Log::Vip, " Y: {}", y);
                                        logln!(Log::Vip, " Pal: {}", pal);
                                        logln!(Log::Vip, " Horizontal flip: {}", horizontal_flip);
                                        logln!(Log::Vip, " Vertical flip: {}", vertical_flip);
                                        logln!(Log::Vip, " Char index: {}", char_index);*/

                                        match eye {
                                            Eye::Left => {
                                                if !l {
                                                    continue;
                                                }
                                            }
                                            Eye::Right => {
                                                if !r {
                                                    continue;
                                                }
                                            }
                                        }

                                        let palette = match pal {
                                            0 => self.reg_jplt0,
                                            1 => self.reg_jplt1,
                                            2 => self.reg_jplt2,
                                            _ => self.reg_jplt3,
                                        };

                                        for offset_y in 0..8 {
                                            let pixel_y = (y as u32).wrapping_add(offset_y);
                                            if pixel_y < block_start_y || pixel_y >= block_end_y {
                                                continue;
                                            }
                                            for offset_x in 0..8 {
                                                let pixel_x = {
                                                    let value = (x as u32).wrapping_add(offset_x);
                                                    match eye {
                                                        Eye::Left => value.wrapping_sub(parallax as u32),
                                                        Eye::Right => value.wrapping_add(parallax as u32),
                                                    }
                                                };
                                                if pixel_x >= FRAMEBUFFER_RESOLUTION_X {
                                                    continue;
                                                }

                                                self.draw_char_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, char_index, horizontal_flip, vertical_flip, palette);
                                            }
                                        }
                                    }
                                }
                                _ => logln!(Log::Vip, "WARNING: Extra obj window found; all obj groups already drawn")
                            }
                        }
                        WindowMode::Affine => {
                            let parallax_x = {
                                match eye {
                                    Eye::Left => (x as u32).wrapping_sub(parallax as u32),
                                    Eye::Right => (x as u32).wrapping_add(parallax as u32),
                                }
                            };

                            for window_y in 0..height {
                                let pixel_y = window_y.wrapping_add(y as u32);
                                if pixel_y < block_start_y || pixel_y >= block_end_y {
                                    continue;
                                }

                                let affine_offset = param_offset + window_y * 16;
                                let affine_bg_x = self.read_vram_halfword(affine_offset) as i16;
                                let affine_bg_parallax = self.read_vram_halfword(affine_offset + 2) as i16;
                                let affine_bg_y = self.read_vram_halfword(affine_offset + 4) as i16;
                                let affine_bg_x_inc = self.read_vram_halfword(affine_offset + 6) as i16;
                                let affine_bg_y_inc = self.read_vram_halfword(affine_offset + 8) as i16;
                                let affine_parallax_x = match eye {
                                    Eye::Left => {
                                        if affine_bg_parallax < 0 {
                                            0u32.wrapping_sub(affine_bg_parallax as u32)
                                        } else {
                                            0
                                        }
                                    }
                                    Eye::Right => {
                                        if affine_bg_parallax > 0 {
                                            0u32.wrapping_add(affine_bg_parallax as u32)
                                        } else {
                                            0
                                        }
                                    }
                                };

                                for window_x in 0..width {
                                    let pixel_x = window_x.wrapping_add(parallax_x);
                                    if pixel_x >= FRAMEBUFFER_RESOLUTION_X {
                                        continue;
                                    }

                                    let parallaxed_window_x = window_x.wrapping_add(affine_parallax_x);

                                    let background_x = (((affine_bg_x as i32) << 6) + ((affine_bg_x_inc as i32) * (parallaxed_window_x as i32)) >> 9) as u32;
                                    let background_y = (((affine_bg_y as i32) << 6) + ((affine_bg_y_inc as i32) * (parallaxed_window_x as i32)) >> 9) as u32;

                                    self.draw_background_pixel(framebuffer_offset, pixel_x, pixel_y, segment_base, segments_x, segments_y, background_x, background_y, overplane, overplane_char_entry);
                                }
                            }
                        }
                        _ => {
                            let parallax_x = {
                                match eye {
                                    Eye::Left => (x as u32).wrapping_sub(parallax as u32),
                                    Eye::Right => (x as u32).wrapping_add(parallax as u32),
                                }
                            };

                            for window_y in 0..height {
                                let pixel_y = window_y.wrapping_add(y as u32);
                                if pixel_y < block_start_y || pixel_y >= block_end_y {
                                    continue;
                                }

                                let line_shift = match mode {
                                    WindowMode::LineShift => {
                                        let line_offset = param_offset + window_y * 4;
                                        let eye_offset = line_offset + match eye {
                                            Eye::Left => 0,
                                            Eye::Right => 2,
                                        };
                                        (self.read_vram_halfword(eye_offset) as i16) as u32
                                    }
                                    _ => 0
                                };

                                for window_x in 0..width {
                                    let pixel_x = window_x.wrapping_add(parallax_x);
                                    if pixel_x >= FRAMEBUFFER_RESOLUTION_X {
                                        continue;
                                    }

                                    let background_x = {
                                        let value = window_x.wrapping_add(bg_x as u32).wrapping_add(line_shift);
                                        match eye {
                                            Eye::Left => value.wrapping_sub(bg_parallax as u32),
                                            Eye::Right => value.wrapping_add(bg_parallax as u32),
                                        }
                                    };
                                    let background_y = window_y.wrapping_add(bg_y as u32);

                                    self.draw_background_pixel(framebuffer_offset, pixel_x, pixel_y, segment_base, segments_x, segments_y, background_x, background_y, overplane, overplane_char_entry);
                                }
                            }
                        }
                    }
                }

                if let WindowMode::Obj = mode {
                    current_obj_group = match current_obj_group {
                        Some(ObjGroup::Group3) => Some(ObjGroup::Group2),
                        Some(ObjGroup::Group2) => Some(ObjGroup::Group1),
                        Some(ObjGroup::Group1) => Some(ObjGroup::Group0),
                        _ => None
                    };
                }
            }

            window_offset -= WINDOW_ENTRY_LENGTH;
            window_index -= 1;
        }
    }

    #[inline(always)]
    fn draw_background_pixel(&mut self, framebuffer_offset: u32, pixel_x: u32, pixel_y: u32, segment_base: u32, segments_x: u32, segments_y: u32, background_x: u32, background_y: u32, overplane: bool, overplane_char_entry: u16) {
        let background_width = segments_x * 512;
        let background_height = segments_y * 512;

        if overplane && (background_x >= background_width || background_y >= background_height) {
            let offset_x = background_x & 0x07;
            let offset_y = background_y & 0x07;

            self.draw_char_entry_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, overplane_char_entry);
        } else {
            let x_segment = (background_x / 512) & (segments_x - 1);
            let y_segment = (background_y / 512) & (segments_y - 1);

            let segment_offset = segment_base + ((y_segment * segments_x) + x_segment) * 0x2000;

            let segment_x = background_x & 0x01ff;
            let segment_y = background_y & 0x01ff;

            self.draw_segment_pixel(framebuffer_offset, pixel_x, pixel_y, segment_offset, segment_x, segment_y);
        }
    }

    #[inline(always)]
    fn draw_segment_pixel(&mut self, framebuffer_offset: u32, pixel_x: u32, pixel_y: u32, segment_offset: u32, segment_x: u32, segment_y: u32) {
        let offset_x = segment_x & 0x07;
        let offset_y = segment_y & 0x07;

        let segment_char_x = segment_x / 8;
        let segment_char_y = segment_y / 8;
        let segment_addr = segment_offset + (segment_char_y * 64 + segment_char_x) * 2;

        let char_entry = self.read_vram_halfword(segment_addr as _);

        self.draw_char_entry_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, char_entry);
    }

    #[inline(always)]
    fn draw_char_entry_pixel(&mut self, framebuffer_offset: u32, pixel_x: u32, pixel_y: u32, offset_x: u32, offset_y: u32, char_entry: u16) {
        let pal = (char_entry >> 14) & 0x03;
        let horizontal_flip = (char_entry & 0x2000) != 0;
        let vertical_flip = (char_entry & 0x1000) != 0;
        let char_index = (char_entry & 0x07ff) as u32;

        let palette = match pal {
            0 => self.reg_gplt0,
            1 => self.reg_gplt1,
            2 => self.reg_gplt2,
            _ => self.reg_gplt3,
        };

        self.draw_char_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, char_index, horizontal_flip, vertical_flip, palette);
    }

    #[inline(always)]
    fn draw_char_pixel(&mut self, framebuffer_offset: u32, pixel_x: u32, pixel_y: u32, offset_x: u32, offset_y: u32, char_index: u32, horizontal_flip: bool, vertical_flip: bool, palette: u8) {
        let offset_x = if horizontal_flip { 7 - offset_x } else { offset_x };
        let offset_y = if vertical_flip { 7 - offset_y } else { offset_y };

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
            return;
        }

        let color = (palette >> (palette_index * 2)) & 0x03;

        let framebuffer_byte_index = (pixel_x * FRAMEBUFFER_RESOLUTION_Y + pixel_y) / 4;
        let framebuffer_byte_shift = (pixel_y & 0x03) * 2;
        let framebuffer_byte_mask = 0x03 << framebuffer_byte_shift;
        let mut framebuffer_byte = self.read_vram_byte(framebuffer_offset + framebuffer_byte_index);
        framebuffer_byte = (framebuffer_byte & !framebuffer_byte_mask) | (color << framebuffer_byte_shift);
        self.write_vram_byte(framebuffer_offset + framebuffer_byte_index, framebuffer_byte);
    }

    fn display(&mut self, video_frame_sink: &mut VideoSink) {
        let left_framebuffer_offset = if self.display_first_framebuffers { 0x00000000 } else { 0x00008000 };
        let right_framebuffer_offset = left_framebuffer_offset + 0x00010000;

        if self.reg_dpctrl_disp && self.reg_dpctrl_synce {
            let mut brightness_1 = (self.reg_brta as u32) * 2;
            let mut brightness_2 = (self.reg_brtb as u32) * 2;
            let mut brightness_3 = ((self.reg_brta as u32) + (self.reg_brtb as u32) + (self.reg_brtc as u32)) * 2;
            if brightness_1 > 255 {
                brightness_1 = 255;
            }
            if brightness_2 > 255 {
                brightness_2 = 255;
            }
            if brightness_3 > 255 {
                brightness_3 = 255;
            }

            for pixel_x in 0..DISPLAY_RESOLUTION_X {
                for pixel_y in 0..DISPLAY_RESOLUTION_Y {
                    let framebuffer_byte_index = (pixel_x * FRAMEBUFFER_RESOLUTION_Y + pixel_y) / 4;
                    let framebuffer_byte_shift = (pixel_y & 0x03) * 2;
                    let left_color = (self.read_vram_byte(left_framebuffer_offset + framebuffer_byte_index) >> framebuffer_byte_shift) & 0x03;
                    let right_color = (self.read_vram_byte(right_framebuffer_offset + framebuffer_byte_index) >> framebuffer_byte_shift) & 0x03;
                    let mut left_brightness = match left_color {
                        0 => 0,
                        1 => brightness_1,
                        2 => brightness_2,
                        _ => brightness_3
                    } as u8;
                    let mut right_brightness = match right_color {
                        0 => 0,
                        1 => brightness_1,
                        2 => brightness_2,
                        _ => brightness_3
                    } as u8;

                    match video_frame_sink.gamma_correction {
                        GammaCorrection::None => (), // Do nothing
                        GammaCorrection::TwoPointTwo => {
                            left_brightness = self.gamma_table[left_brightness as usize];
                            right_brightness = self.gamma_table[right_brightness as usize];
                        }
                    }

                    let buffer_index = pixel_y * DISPLAY_RESOLUTION_X + pixel_x;

                    match video_frame_sink.buffer {
                        PixelBuffer::Xrgb1555(ref mut buffer) => {
                            match video_frame_sink.format {
                                StereoVideoFormat::AnaglyphRedElectricCyan => {
                                    let red = (left_brightness as u16) >> 3;
                                    let cyan = (right_brightness as u16) >> 3;

                                    buffer[buffer_index as usize] = (red << 10) | (cyan << 5) | cyan;
                                }
                            }
                        }
                        PixelBuffer::Rgb565(ref mut buffer) => {
                            match video_frame_sink.format {
                                StereoVideoFormat::AnaglyphRedElectricCyan => {
                                    let red = (left_brightness as u16) >> 3;
                                    let green = (right_brightness as u16) >> 2;
                                    let blue = (right_brightness as u16) >> 3;

                                    buffer[buffer_index as usize] = (red << 11) | (green << 5) | blue;
                                }
                            }
                        }
                        PixelBuffer::Xrgb8888(ref mut buffer) => {
                            match video_frame_sink.format {
                                StereoVideoFormat::AnaglyphRedElectricCyan => {
                                    let red = left_brightness as u32;
                                    let cyan = right_brightness as u32;

                                    buffer[buffer_index as usize] = (red << 16) | (cyan << 8) | cyan;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            match video_frame_sink.buffer {
                PixelBuffer::Xrgb1555(ref mut buffer) => {
                    for pixel in buffer.iter_mut() {
                        *pixel = 0;
                    }
                }
                PixelBuffer::Rgb565(ref mut buffer) => {
                    for pixel in buffer.iter_mut() {
                        *pixel = 0;
                    }
                }
                PixelBuffer::Xrgb8888(ref mut buffer) => {
                    for pixel in buffer.iter_mut() {
                        *pixel = 0;
                    }
                }
            }
        }

        video_frame_sink.is_populated = true;
    }
}
