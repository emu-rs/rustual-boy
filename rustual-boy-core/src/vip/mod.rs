mod mem_map;

use sinks::*;

use self::mem_map::*;

const FRAMEBUFFER_RESOLUTION_X: usize = 384;
const FRAMEBUFFER_RESOLUTION_Y: usize = 256;

pub const DISPLAY_RESOLUTION_X: usize = 384;
pub const DISPLAY_RESOLUTION_Y: usize = 224;

const DRAWING_BLOCK_HEIGHT: usize = 8;
const DRAWING_BLOCK_COUNT: usize = DISPLAY_RESOLUTION_Y / DRAWING_BLOCK_HEIGHT;

// 20mhz / (1s / 5ms) = 100000 clocks
const DISPLAY_FRAME_QUARTER_PERIOD: usize = 100000;

const DRAWING_PERIOD: usize = DISPLAY_FRAME_QUARTER_PERIOD * 2;
const DRAWING_BLOCK_PERIOD: usize = DRAWING_PERIOD / DRAWING_BLOCK_COUNT;

// 20mhz / (1s / 56us) = 1120 clocks
const DRAWING_SBOUT_PERIOD: usize = 1120;

enum DisplayState {
    Idle,
    LeftFramebuffer,
    RightFramebuffer,
    Finished,
}

enum DrawingState {
    Idle,
    Drawing,
}

enum Eye {
    Left,
    Right,
}

#[derive(PartialEq, Eq)]
enum WindowMode {
    Normal,
    LineShift,
    Affine,
    Obj,
}

#[derive(Debug, Clone, Copy)]
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

    reg_interrupt_pending_left_display_finished: bool,
    reg_interrupt_pending_right_display_finished: bool,
    reg_interrupt_pending_start_of_game_frame: bool,
    reg_interrupt_pending_start_of_display_frame: bool,
    reg_interrupt_pending_sbhit: bool,
    reg_interrupt_pending_drawing_finished: bool,

    reg_interrupt_enable_left_display_finished: bool,
    reg_interrupt_enable_right_display_finished: bool,
    reg_interrupt_enable_start_of_game_frame: bool,
    reg_interrupt_enable_start_of_display_frame: bool,
    reg_interrupt_enable_sbhit: bool,
    reg_interrupt_enable_drawing_finished: bool,

    reg_display_control_display_enable: bool,
    reg_display_control_sync_enable: bool,

    reg_drawing_control_drawing_enable: bool,
    reg_drawing_control_sbcount: usize,
    reg_drawing_control_sbcmp: usize,
    reg_drawing_control_sbout: bool,

    reg_game_frame_control: usize,

    reg_led_brightness_1: u8,
    reg_led_brightness_2: u8,
    reg_led_brightness_3: u8,

    reg_obj_group_0_ptr: u16,
    reg_obj_group_1_ptr: u16,
    reg_obj_group_2_ptr: u16,
    reg_obj_group_3_ptr: u16,

    reg_bg_palette_0: u8,
    reg_bg_palette_1: u8,
    reg_bg_palette_2: u8,
    reg_bg_palette_3: u8,
    reg_obj_palette_0: u8,
    reg_obj_palette_1: u8,
    reg_obj_palette_2: u8,
    reg_obj_palette_3: u8,

    reg_clear_color: u8,

    display_frame_quarter_clock_counter: usize,
    display_frame_quarter_counter: usize,

    drawing_block_counter: usize,
    drawing_sbout_counter: usize,

    game_frame_counter: usize,

    display_first_framebuffers: bool,
    last_clear_color: u8,

    gamma_table: Box<[u8; 256]>,
}

impl Vip {
    pub fn new() -> Vip {
        let mut vram = vec![0; VRAM_LENGTH as usize].into_boxed_slice();
        let vram_ptr = vram.as_mut_ptr();

        let gamma = 2.2;
        let mut gamma_table = Box::new([0; 256]);
        for (i, entry) in gamma_table.iter_mut().enumerate() {
            let mut value = (((i as f64) / 255.0).powf(1.0 / gamma) * 255.0) as isize;
            if value < 0 {
                value = 0;
            }
            if value > 255 {
                value = 0;
            }

            *entry = value as u8;
        }

        Vip {
            _vram: vram,
            vram_ptr: vram_ptr,

            display_state: DisplayState::Idle,

            drawing_state: DrawingState::Idle,

            reg_interrupt_pending_left_display_finished: false,
            reg_interrupt_pending_right_display_finished: false,
            reg_interrupt_pending_start_of_game_frame: false,
            reg_interrupt_pending_start_of_display_frame: false,
            reg_interrupt_pending_sbhit: false,
            reg_interrupt_pending_drawing_finished: false,

            reg_interrupt_enable_left_display_finished: false,
            reg_interrupt_enable_right_display_finished: false,
            reg_interrupt_enable_start_of_game_frame: false,
            reg_interrupt_enable_start_of_display_frame: false,
            reg_interrupt_enable_sbhit: false,
            reg_interrupt_enable_drawing_finished: false,

            reg_display_control_display_enable: false,
            reg_display_control_sync_enable: false,

            reg_drawing_control_drawing_enable: false,
            reg_drawing_control_sbcount: 0,
            reg_drawing_control_sbcmp: 0,
            reg_drawing_control_sbout: false,

            reg_game_frame_control: 1,

            reg_led_brightness_1: 0,
            reg_led_brightness_2: 0,
            reg_led_brightness_3: 0,

            reg_obj_group_0_ptr: 0,
            reg_obj_group_1_ptr: 0,
            reg_obj_group_2_ptr: 0,
            reg_obj_group_3_ptr: 0,

            reg_bg_palette_0: 0,
            reg_bg_palette_1: 0,
            reg_bg_palette_2: 0,
            reg_bg_palette_3: 0,
            reg_obj_palette_0: 0,
            reg_obj_palette_1: 0,
            reg_obj_palette_2: 0,
            reg_obj_palette_3: 0,

            reg_clear_color: 0,

            display_frame_quarter_clock_counter: 0,
            display_frame_quarter_counter: 0,

            drawing_block_counter: 0,
            drawing_sbout_counter: 0,

            game_frame_counter: 0,

            display_first_framebuffers: false,
            last_clear_color: 0,

            gamma_table: gamma_table,
        }
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
            INTERRUPT_PENDING_REG => {
                //logln!("WARNING: Read halfword from Interrupt Pending Reg not fully implemented");
                (if self.reg_interrupt_pending_left_display_finished { 1 } else { 0 } << 1) |
                (if self.reg_interrupt_pending_right_display_finished { 1 } else { 0 } << 2) |
                (if self.reg_interrupt_pending_start_of_game_frame { 1 } else { 0 } << 3) |
                (if self.reg_interrupt_pending_start_of_display_frame { 1 } else { 0 } << 4) |
                (if self.reg_interrupt_pending_sbhit { 1 } else { 0 } << 13) |
                (if self.reg_interrupt_pending_drawing_finished { 1 } else { 0 } << 14)
            }
            INTERRUPT_ENABLE_REG => {
                logln!("WARNING: Read halfword from Interrupt Enable Reg not fully implemented");
                (if self.reg_interrupt_enable_left_display_finished { 1 } else { 0 } << 1) |
                (if self.reg_interrupt_enable_right_display_finished { 1 } else { 0 } << 2) |
                (if self.reg_interrupt_enable_start_of_game_frame { 1 } else { 0 } << 3) |
                (if self.reg_interrupt_enable_start_of_display_frame { 1 } else { 0 } << 4) |
                (if self.reg_interrupt_enable_sbhit { 1 } else { 0 } << 13) |
                (if self.reg_interrupt_enable_drawing_finished { 1 } else { 0 } << 14)
            }
            INTERRUPT_CLEAR_REG => {
                logln!("WARNING: Attempted read halfword from Interrupt Clear Reg");
                0
            }
            DISPLAY_CONTROL_READ_REG => {
                let scan_ready = true; // TODO
                // TODO: Not entirely sure this is correct
                let frame_clock = match self.display_state {
                    DisplayState::Idle => true,
                    _ => false
                };
                let mem_refresh = false; // TODO
                let column_table_addr_lock = false; // TODO

                (if self.reg_display_control_display_enable { 1 } else { 0 } << 1) |
                (match self.display_state {
                    DisplayState::Idle | DisplayState::Finished => 0b0000,
                    DisplayState::LeftFramebuffer => if self.display_first_framebuffers { 0b0001 } else { 0b0100 },
                    DisplayState::RightFramebuffer => if self.display_first_framebuffers { 0b0010 } else { 0b1000 },
                } << 2) |
                (if scan_ready { 1 } else { 0 } << 6) |
                (if frame_clock { 1 } else { 0 } << 7) |
                (if mem_refresh { 1 } else { 0 } << 8) |
                (if self.reg_display_control_sync_enable { 1 } else { 0 } << 9) |
                (if column_table_addr_lock { 1 } else { 0 } << 10)
            }
            DISPLAY_CONTROL_WRITE_REG => {
                logln!("WARNING: Attempted read halfword from Display Control Write Reg");
                0
            }
            LED_BRIGHTNESS_1_REG => self.reg_led_brightness_1 as _,
            LED_BRIGHTNESS_2_REG => self.reg_led_brightness_2 as _,
            LED_BRIGHTNESS_3_REG => self.reg_led_brightness_3 as _,
            LED_BRIGHTNESS_IDLE_REG => {
                logln!("WARNING: Read halfword from LED Brightness Idle Reg not yet implemented");
                0
            }
            GAME_FRAME_CONTROL_REG => {
                (self.reg_game_frame_control - 1) as u16
            }
            DRAWING_CONTROL_READ_REG => {
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

                (if self.reg_drawing_control_drawing_enable { 1 } else { 0 } << 1) |
                (if drawing_to_frame_buffer_0 { 1 } else { 0 } << 2) |
                (if drawing_to_frame_buffer_1 { 1 } else { 0 } << 3) |
                (if drawing_exceeds_frame_period { 1 } else { 0 } << 4) |
                ((self.reg_drawing_control_sbcount as u16) << 8) |
                (if self.reg_drawing_control_sbout { 1 } else { 0 } << 15)
            }
            DRAWING_CONTROL_WRITE_REG => {
                logln!("WARNING: Attempted read halfword from Drawing Control Write Reg");
                0
            }
            OBJ_GROUP_0_POINTER_REG => self.reg_obj_group_0_ptr,
            OBJ_GROUP_1_POINTER_REG => self.reg_obj_group_1_ptr,
            OBJ_GROUP_2_POINTER_REG => self.reg_obj_group_2_ptr,
            OBJ_GROUP_3_POINTER_REG => self.reg_obj_group_3_ptr,
            BG_PALETTE_0_REG => self.reg_bg_palette_0 as _,
            BG_PALETTE_1_REG => self.reg_bg_palette_1 as _,
            BG_PALETTE_2_REG => self.reg_bg_palette_2 as _,
            BG_PALETTE_3_REG => self.reg_bg_palette_3 as _,
            OBJ_PALETTE_0_REG => self.reg_obj_palette_0 as _,
            OBJ_PALETTE_1_REG => self.reg_obj_palette_1 as _,
            OBJ_PALETTE_2_REG => self.reg_obj_palette_2 as _,
            OBJ_PALETTE_3_REG => self.reg_obj_palette_3 as _,
            CLEAR_COLOR_REG => self.reg_clear_color as _,
            CHR_RAM_PATTERN_TABLE_0_MIRROR_START ... CHR_RAM_PATTERN_TABLE_0_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_0_MIRROR_START + CHR_RAM_PATTERN_TABLE_0_START),
            CHR_RAM_PATTERN_TABLE_1_MIRROR_START ... CHR_RAM_PATTERN_TABLE_1_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_1_MIRROR_START + CHR_RAM_PATTERN_TABLE_1_START),
            CHR_RAM_PATTERN_TABLE_2_MIRROR_START ... CHR_RAM_PATTERN_TABLE_2_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_2_MIRROR_START + CHR_RAM_PATTERN_TABLE_2_START),
            CHR_RAM_PATTERN_TABLE_3_MIRROR_START ... CHR_RAM_PATTERN_TABLE_3_MIRROR_END =>
                self.read_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_3_MIRROR_START + CHR_RAM_PATTERN_TABLE_3_START),
            _ => {
                logln!("WARNING: Attempted read halfword from unrecognized VIP address (addr: 0x{:08x})", addr);
                0
            }
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0x0007ffff;
        let addr = addr & 0xfffffffe;
        match addr {
            VRAM_START ... VRAM_END => self.write_vram_halfword(addr - VRAM_START, value),
            INTERRUPT_PENDING_REG => {
                logln!("WARNING: Attempted write halfword to Interrupt Pending Reg");
            }
            INTERRUPT_ENABLE_REG => {
                logln!("WARNING: Write halfword to Interrupt Enable Reg not fully implemented (value: 0x{:04x})", value);
                self.reg_interrupt_enable_left_display_finished = (value & 0x0002) != 0;
                self.reg_interrupt_enable_right_display_finished = (value & 0x0004) != 0;
                self.reg_interrupt_enable_start_of_game_frame = (value & 0x0008) != 0;
                self.reg_interrupt_enable_start_of_display_frame = (value & 0x0010) != 0;
                self.reg_interrupt_enable_sbhit = (value & 0x2000) != 0;
                self.reg_interrupt_enable_drawing_finished = (value & 0x4000) != 0;
            }
            INTERRUPT_CLEAR_REG => {
                logln!("WARNING: Write halfword to Interrupt Clear Reg not fully implemented (value: 0x{:04x})", value);
                if (value & 0x0002) != 0 {
                    self.reg_interrupt_pending_left_display_finished = false;
                }
                if (value & 0x0004) != 0 {
                    self.reg_interrupt_pending_right_display_finished = false;
                }
                if (value & 0x0008) != 0 {
                    self.reg_interrupt_pending_start_of_game_frame = false;
                }
                if (value & 0x0010) != 0 {
                    self.reg_interrupt_pending_start_of_display_frame = false;
                }
                if (value & 0x2000) != 0 {
                    self.reg_interrupt_pending_sbhit = false;
                }
                if (value & 0x4000) != 0 {
                    self.reg_interrupt_pending_drawing_finished = false;
                }
            }
            DISPLAY_CONTROL_READ_REG => {
                logln!("WARNING: Attempted write halfword to Display Control Read Reg");
            }
            DISPLAY_CONTROL_WRITE_REG => {
                logln!("WARNING: Write halfword to Display Control Write Reg not fully implemented (value: 0x{:04x})", value);

                let reset = (value & 0x0001) != 0;
                let enable = (value & 0x0002) != 0;
                let _mem_refresh = (value & 0x0100) != 0; // TODO
                self.reg_display_control_sync_enable = (value & 0x0200) != 0;
                let _column_table_addr_lock = (value & 0x0400) != 0;

                if reset {
                    self.display_state = DisplayState::Finished;

                    self.reg_interrupt_pending_start_of_game_frame = false;
                    self.reg_interrupt_pending_start_of_display_frame = false;
                    self.reg_interrupt_pending_left_display_finished = false;
                    self.reg_interrupt_pending_right_display_finished = false;
                } else if enable && !self.reg_display_control_display_enable {
                    self.display_state = DisplayState::Finished;
                }

                self.reg_display_control_display_enable = enable;
            }
            LED_BRIGHTNESS_1_REG => self.reg_led_brightness_1 = value as _,
            LED_BRIGHTNESS_2_REG => self.reg_led_brightness_2 = value as _,
            LED_BRIGHTNESS_3_REG => self.reg_led_brightness_3 = value as _,
            LED_BRIGHTNESS_IDLE_REG => {
                logln!("WARNING: Write halfword to LED Brightness Idle Reg not yet implemented (value: 0x{:04x})", value);
            }
            GAME_FRAME_CONTROL_REG => {
                logln!("Game Frame Control written (value: 0x{:04x})", value);
                self.reg_game_frame_control = (value as usize) + 1;
            }
            DRAWING_CONTROL_READ_REG => {
                logln!("WARNING: Attempted write halfword to Drawing Control Read Reg (value: 0x{:04x})", value);
            }
            DRAWING_CONTROL_WRITE_REG => {
                logln!("WARNING: Write halfword to Drawing Control Write Reg not fully implemented (value: 0x{:04x})", value);

                let reset = (value & 0x01) != 0;
                self.reg_drawing_control_drawing_enable = (value & 0x02) != 0;
                self.reg_drawing_control_sbcmp = ((value as usize) >> 8) & 0x1f;

                if reset {
                    self.drawing_state = DrawingState::Idle;

                    self.reg_interrupt_pending_sbhit = false;
                    self.reg_interrupt_pending_drawing_finished = false;
                }
            }
            OBJ_GROUP_0_POINTER_REG => self.reg_obj_group_0_ptr = value & 0x03ff,
            OBJ_GROUP_1_POINTER_REG => self.reg_obj_group_1_ptr = value & 0x03ff,
            OBJ_GROUP_2_POINTER_REG => self.reg_obj_group_2_ptr = value & 0x03ff,
            OBJ_GROUP_3_POINTER_REG => self.reg_obj_group_3_ptr = value & 0x03ff,
            BG_PALETTE_0_REG => self.reg_bg_palette_0 = value as _,
            BG_PALETTE_1_REG => self.reg_bg_palette_1 = value as _,
            BG_PALETTE_2_REG => self.reg_bg_palette_2 = value as _,
            BG_PALETTE_3_REG => self.reg_bg_palette_3 = value as _,
            OBJ_PALETTE_0_REG => self.reg_obj_palette_0 = value as _,
            OBJ_PALETTE_1_REG => self.reg_obj_palette_1 = value as _,
            OBJ_PALETTE_2_REG => self.reg_obj_palette_2 = value as _,
            OBJ_PALETTE_3_REG => self.reg_obj_palette_3 = value as _,
            CLEAR_COLOR_REG => self.reg_clear_color = (value & 0x03) as _,
            CHR_RAM_PATTERN_TABLE_0_MIRROR_START ... CHR_RAM_PATTERN_TABLE_0_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_0_MIRROR_START + CHR_RAM_PATTERN_TABLE_0_START, value),
            CHR_RAM_PATTERN_TABLE_1_MIRROR_START ... CHR_RAM_PATTERN_TABLE_1_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_1_MIRROR_START + CHR_RAM_PATTERN_TABLE_1_START, value),
            CHR_RAM_PATTERN_TABLE_2_MIRROR_START ... CHR_RAM_PATTERN_TABLE_2_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_2_MIRROR_START + CHR_RAM_PATTERN_TABLE_2_START, value),
            CHR_RAM_PATTERN_TABLE_3_MIRROR_START ... CHR_RAM_PATTERN_TABLE_3_MIRROR_END =>
                self.write_vram_halfword(addr - CHR_RAM_PATTERN_TABLE_3_MIRROR_START + CHR_RAM_PATTERN_TABLE_3_START, value),
            _ => {
                logln!("WARNING: Attempted write halfword to unrecognized VIP address (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
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

    pub fn cycles(&mut self, cycles: usize, video_frame_sink: &mut Sink<VideoFrame>) -> bool {
        let mut raise_interrupt = false;

        for _ in 0..cycles {
            self.display_frame_quarter_clock_counter += 1;
            if self.display_frame_quarter_clock_counter >= DISPLAY_FRAME_QUARTER_PERIOD {
                self.display_frame_quarter_clock_counter = 0;

                match self.display_frame_quarter_counter {
                    0 => {
                        self.frame_clock(&mut raise_interrupt);
                    }
                    1 => {
                        self.display(video_frame_sink);

                        if self.reg_display_control_display_enable && self.reg_display_control_sync_enable {
                            self.begin_left_framebuffer_display_process();
                        }
                    }
                    2 => {
                        if self.reg_display_control_display_enable {
                            if let DisplayState::LeftFramebuffer = self.display_state {
                                self.reg_interrupt_pending_left_display_finished = true;
                                if self.reg_interrupt_enable_left_display_finished {
                                    raise_interrupt = true;
                                }
                            }

                            if self.reg_display_control_sync_enable {
                                self.begin_right_framebuffer_display_process();
                            }
                        }
                    }
                    _ => {
                        if self.reg_display_control_display_enable {
                            if let DisplayState::RightFramebuffer = self.display_state {
                                self.reg_interrupt_pending_right_display_finished = true;
                                if self.reg_interrupt_enable_right_display_finished {
                                    raise_interrupt = true;
                                }
                            }

                            self.end_display_process();
                        }
                    }
                }

                self.display_frame_quarter_counter = match self.display_frame_quarter_counter {
                    3 => 0,
                    _ => self.display_frame_quarter_counter + 1
                };
            }

            if let DrawingState::Drawing = self.drawing_state {
                self.drawing_block_counter += 1;
                if self.drawing_block_counter >= DRAWING_BLOCK_PERIOD {
                    self.drawing_block_counter = 0;

                    if self.reg_drawing_control_sbcount < DRAWING_BLOCK_COUNT {
                        self.end_drawing_block(&mut raise_interrupt);

                        if self.reg_drawing_control_sbcount < DRAWING_BLOCK_COUNT - 1 {
                            self.reg_drawing_control_sbcount += 1;
                            if self.reg_drawing_control_drawing_enable {
                                self.begin_drawing_block();
                            }
                        } else {
                            self.reg_drawing_control_sbcount = 0;

                            self.end_drawing_process();
                            self.reg_interrupt_pending_drawing_finished = true;
                            if self.reg_interrupt_enable_drawing_finished {
                                raise_interrupt = true;
                            }
                        }
                    }
                }

                if self.reg_drawing_control_sbout {
                    self.drawing_sbout_counter += 1;
                    if self.drawing_sbout_counter >= DRAWING_SBOUT_PERIOD {
                        self.reg_drawing_control_sbout = false;
                    }
                }
            }
        }

        raise_interrupt
    }

    fn frame_clock(&mut self, raise_interrupt: &mut bool) {
        logln!("Frame clock rising edge");

        if self.reg_display_control_display_enable {
            self.reg_interrupt_pending_start_of_display_frame = true;
            if self.reg_interrupt_enable_start_of_display_frame {
                *raise_interrupt = true;
            }

            self.begin_display_process();
        }

        self.game_frame_counter += 1;
        if self.game_frame_counter >= self.reg_game_frame_control {
            self.game_frame_counter = 0;
            self.game_clock(raise_interrupt);
        }
    }

    fn game_clock(&mut self, raise_interrupt: &mut bool) {
        logln!("Game clock rising edge");

        self.reg_interrupt_pending_start_of_game_frame = true;
        if self.reg_interrupt_enable_start_of_game_frame {
            *raise_interrupt = true;
        }

        if self.reg_drawing_control_drawing_enable {
            self.display_first_framebuffers = !self.display_first_framebuffers;

            self.begin_drawing_process();
        } else {
            self.reg_interrupt_pending_drawing_finished = true;
            if self.reg_interrupt_enable_drawing_finished {
                *raise_interrupt = true;
            }
        }
    }

    fn begin_drawing_process(&mut self) {
        logln!("Begin drawing process");
        self.drawing_state = DrawingState::Drawing;

        self.reg_drawing_control_sbcount = 0;

        self.drawing_block_counter = 0;

        self.begin_drawing_block();
    }

    fn begin_drawing_block(&mut self) {
        logln!("Begin drawing block {}", self.reg_drawing_control_sbcount);
    }

    fn end_drawing_block(&mut self, raise_interrupt: &mut bool) {
        logln!("End drawing block {}", self.reg_drawing_control_sbcount);

        self.draw_current_block();

        if self.reg_drawing_control_sbcount == self.reg_drawing_control_sbcmp {
            self.reg_drawing_control_sbout = true;
            self.drawing_sbout_counter = 0;

            self.reg_interrupt_pending_sbhit = true;
            if self.reg_interrupt_enable_sbhit {
                *raise_interrupt = true;
            }
        }
    }

    fn end_drawing_process(&mut self) {
        logln!("End drawing process");
        self.drawing_state = DrawingState::Idle;
    }

    fn begin_display_process(&mut self) {
        logln!("Start display process");
        self.display_state = DisplayState::Idle;
    }

    fn begin_left_framebuffer_display_process(&mut self) {
        logln!("Start left framebuffer display process");
        self.display_state = DisplayState::LeftFramebuffer;
    }

    fn begin_right_framebuffer_display_process(&mut self) {
        logln!("Start right framebuffer display process");
        self.display_state = DisplayState::RightFramebuffer;
    }

    fn end_display_process(&mut self) {
        logln!("End display process");
        self.display_state = DisplayState::Finished;
    }

    fn draw_current_block(&mut self) {
        let draw_to_first_framebuffers = !self.display_first_framebuffers;
        let left_framebuffer_offset = if draw_to_first_framebuffers { 0x00000000 } else { 0x00008000 };
        let right_framebuffer_offset = left_framebuffer_offset + 0x00010000;

        let block_start_y = (self.reg_drawing_control_sbcount as u32) * 8;
        let block_end_y = block_start_y + (DRAWING_BLOCK_HEIGHT as u32);

        let clear_pixels = (self.last_clear_color << 6) | (self.last_clear_color << 4) | (self.last_clear_color << 2) | self.last_clear_color;
        for x in 0..FRAMEBUFFER_RESOLUTION_X as u32 {
            let column_offset = (x * (FRAMEBUFFER_RESOLUTION_Y as u32) + block_start_y) / 4;
            self.write_vram_byte(left_framebuffer_offset + column_offset, clear_pixels);
            self.write_vram_byte(left_framebuffer_offset + column_offset + 1, clear_pixels);
            self.write_vram_byte(right_framebuffer_offset + column_offset, clear_pixels);
            self.write_vram_byte(right_framebuffer_offset + column_offset + 1, clear_pixels);
        }
        // Latch clear color reg _after_ each block. This is a known (and documented) hardware bug.
        self.last_clear_color = self.reg_clear_color;

        let mut current_obj_group = Some(ObjGroup::Group3);

        const WINDOW_ENTRY_LENGTH: u32 = 32;
        let mut window_offset = WINDOW_ATTRIBS_END + 1 - WINDOW_ENTRY_LENGTH;
        let mut window_index = 31;
        for _ in 0..32 {
            logln!("Window {}", window_index);

            let header = self.read_vram_halfword(window_offset);
            logln!(" Header: 0x{:04x}", header);

            if header == 0 {
                logln!("  [Dummy world]");
            } else {
                let base = (header & 0x000f) as u32;
                let stop = (header & 0x0040) != 0;
                let out_of_bounds = (header & 0x0080) != 0;
                let bg_height = ((header >> 8) & 0x03) as u32;
                let bg_width = ((header >> 10) & 0x03) as u32;
                let mode = ((header >> 12) & 0x03) as usize;
                let right_on = (header & 0x4000) != 0;
                let left_on = (header & 0x8000) != 0;
                /*logln!("  base: 0x{:02x}", base);
                logln!("  stop: {}", stop);
                logln!("  out of bounds: {}", out_of_bounds);
                logln!("  w, h: {}, {}", bg_width, bg_height);
                logln!("  mode: {}", mode);
                logln!("  l, r: {}, {}", left_on, right_on);*/

                let x = self.read_vram_halfword(window_offset + 2) as i16;
                let parallax = self.read_vram_halfword(window_offset + 4) as i16;
                let y = self.read_vram_halfword(window_offset + 6) as i16;
                let bg_x = self.read_vram_halfword(window_offset + 8) as i16;
                let bg_parallax = self.read_vram_halfword(window_offset + 10) as i16;
                let bg_y = self.read_vram_halfword(window_offset + 12) as i16;
                let width = self.read_vram_halfword(window_offset + 14);
                let height = self.read_vram_halfword(window_offset + 16);
                let param_base = self.read_vram_halfword(window_offset + 18) as u32;
                let out_of_bounds_char = self.read_vram_halfword(window_offset + 20);
                /*logln!(" X: {}", x);
                logln!(" Parallax: {}", parallax);
                logln!(" Y: {}", y);
                logln!(" BG X: {}", bg_x);
                logln!(" BG Parallax: {}", bg_parallax);
                logln!(" BG Y: {}", bg_y);
                logln!(" Width: {}", width);
                logln!(" Height: {}", height);
                logln!(" Param base: 0x{:04x}", param_base);
                logln!(" Out of bounds char: 0x{:04x}", out_of_bounds_char);*/

                if stop {
                    break;
                }

                let width = (width as u32) + 1;
                let height = (height as u32) + 1;
                let segment_base = 0x00020000 + base * 0x00002000;
                let segments_x = 1 << bg_width;
                let segments_y = 1 << bg_height;
                let param_offset = 0x00020000 + param_base * 2;
                let out_of_bounds_char_entry = self.read_vram_halfword(0x00020000 + (out_of_bounds_char as u32) * 2);

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
                            //logln!("Current obj group: {:?}", current_obj_group);

                            match current_obj_group {
                                Some(obj_group) => {
                                    let starting_obj_index = match obj_group {
                                        ObjGroup::Group0 => self.reg_obj_group_0_ptr,
                                        ObjGroup::Group1 => self.reg_obj_group_1_ptr,
                                        ObjGroup::Group2 => self.reg_obj_group_2_ptr,
                                        ObjGroup::Group3 => self.reg_obj_group_3_ptr,
                                    };
                                    let mut ending_obj_index = match obj_group {
                                        ObjGroup::Group0 => 0,
                                        ObjGroup::Group1 => self.reg_obj_group_0_ptr + 1,
                                        ObjGroup::Group2 => self.reg_obj_group_1_ptr + 1,
                                        ObjGroup::Group3 => self.reg_obj_group_2_ptr + 1,
                                    };
                                    if ending_obj_index >= starting_obj_index {
                                        ending_obj_index = 0;
                                    }
                                    for i in (ending_obj_index..starting_obj_index + 1).rev() {
                                        //logln!("Current obj: {}", i);

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
                                        /*logln!(" X: {}", x);
                                        logln!(" L: {}", l);
                                        logln!(" R: {}", r);
                                        logln!(" Parallax: {}", parallax);
                                        logln!(" Y: {}", y);
                                        logln!(" Pal: {}", pal);
                                        logln!(" Horizontal flip: {}", horizontal_flip);
                                        logln!(" Vertical flip: {}", vertical_flip);
                                        logln!(" Char index: {}", char_index);*/

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
                                            0 => self.reg_obj_palette_0,
                                            1 => self.reg_obj_palette_1,
                                            2 => self.reg_obj_palette_2,
                                            _ => self.reg_obj_palette_3
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
                                                if pixel_x >= FRAMEBUFFER_RESOLUTION_X as u32 {
                                                    continue;
                                                }

                                                self.draw_char_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, char_index, horizontal_flip, vertical_flip, palette);
                                            }
                                        }
                                    }
                                }
                                _ => logln!("WARNING: Extra obj window found; all obj groups already drawn")
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
                                    if pixel_x >= FRAMEBUFFER_RESOLUTION_X as u32 {
                                        continue;
                                    }

                                    let parallaxed_window_x = window_x.wrapping_add(affine_parallax_x);

                                    let background_x = (((affine_bg_x as i32) << 6) + ((affine_bg_x_inc as i32) * (parallaxed_window_x as i32)) >> 9) as u32;
                                    let background_y = (((affine_bg_y as i32) << 6) + ((affine_bg_y_inc as i32) * (parallaxed_window_x as i32)) >> 9) as u32;

                                    self.draw_background_pixel(framebuffer_offset, pixel_x, pixel_y, segment_base, segments_x, segments_y, background_x, background_y, out_of_bounds, out_of_bounds_char_entry);
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
                                    if pixel_x >= FRAMEBUFFER_RESOLUTION_X as u32 {
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

                                    self.draw_background_pixel(framebuffer_offset, pixel_x, pixel_y, segment_base, segments_x, segments_y, background_x, background_y, out_of_bounds, out_of_bounds_char_entry);
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
    fn draw_background_pixel(&mut self, framebuffer_offset: u32, pixel_x: u32, pixel_y: u32, segment_base: u32, segments_x: u32, segments_y: u32, background_x: u32, background_y: u32, out_of_bounds: bool, out_of_bounds_char_entry: u16) {
        let background_width = segments_x * 512;
        let background_height = segments_y * 512;

        if out_of_bounds && (background_x >= background_width || background_y >= background_height) {
            let offset_x = background_x & 0x07;
            let offset_y = background_y & 0x07;

            self.draw_char_entry_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, out_of_bounds_char_entry);
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
            0 => self.reg_bg_palette_0,
            1 => self.reg_bg_palette_1,
            2 => self.reg_bg_palette_2,
            _ => self.reg_bg_palette_3
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

        let framebuffer_byte_index = (pixel_x * (FRAMEBUFFER_RESOLUTION_Y as u32) + pixel_y) / 4;
        let framebuffer_byte_shift = (pixel_y & 0x03) * 2;
        let framebuffer_byte_mask = 0x03 << framebuffer_byte_shift;
        let mut framebuffer_byte = self.read_vram_byte(framebuffer_offset + framebuffer_byte_index);
        framebuffer_byte = (framebuffer_byte & !framebuffer_byte_mask) | (color << framebuffer_byte_shift);
        self.write_vram_byte(framebuffer_offset + framebuffer_byte_index, framebuffer_byte);
    }

    fn display(&mut self, video_frame_sink: &mut Sink<VideoFrame>) {
        let left_framebuffer_offset = if self.display_first_framebuffers { 0x00000000 } else { 0x00008000 };
        let right_framebuffer_offset = left_framebuffer_offset + 0x00010000;

        let mut left_buffer = vec![0; DISPLAY_RESOLUTION_X * DISPLAY_RESOLUTION_Y].into_boxed_slice();
        let mut right_buffer = vec![0; DISPLAY_RESOLUTION_X * DISPLAY_RESOLUTION_Y].into_boxed_slice();
        let left_buffer_ptr = left_buffer.as_mut_ptr();
        let right_buffer_ptr = right_buffer.as_mut_ptr();

        if self.reg_display_control_display_enable && self.reg_display_control_sync_enable {
            let mut brightness_1_index = (self.reg_led_brightness_1 as usize) * 2;
            let mut brightness_2_index = (self.reg_led_brightness_2 as usize) * 2;
            let mut brightness_3_index = ((self.reg_led_brightness_1 as usize) + (self.reg_led_brightness_2 as usize) + (self.reg_led_brightness_3 as usize)) * 2;
            if brightness_1_index > 255 {
                brightness_1_index = 255;
            }
            if brightness_2_index > 255 {
                brightness_2_index = 255;
            }
            if brightness_3_index > 255 {
                brightness_3_index = 255;
            }
            let brightness_1 = self.gamma_table[brightness_1_index] as u32;
            let brightness_2 = self.gamma_table[brightness_2_index] as u32;
            let brightness_3 = self.gamma_table[brightness_3_index] as u32;

            unsafe {
                for pixel_x in 0..DISPLAY_RESOLUTION_X as u32 {
                    for pixel_y in 0..DISPLAY_RESOLUTION_Y as u32 {
                        let framebuffer_byte_index = (pixel_x * (FRAMEBUFFER_RESOLUTION_Y as u32) + pixel_y) / 4;
                        let framebuffer_byte_shift = (pixel_y & 0x03) * 2;
                        let left_color = (self.read_vram_byte(left_framebuffer_offset + framebuffer_byte_index) >> framebuffer_byte_shift) & 0x03;
                        let right_color = (self.read_vram_byte(right_framebuffer_offset + framebuffer_byte_index) >> framebuffer_byte_shift) & 0x03;
                        let left_brightness = match left_color {
                            0 => 0,
                            1 => brightness_1,
                            2 => brightness_2,
                            _ => brightness_3
                        } as u8;
                        let right_brightness = match right_color {
                            0 => 0,
                            1 => brightness_1,
                            2 => brightness_2,
                            _ => brightness_3
                        } as u8;
                        let buffer_index = pixel_y * (DISPLAY_RESOLUTION_X as u32) + pixel_x;
                        *left_buffer_ptr.offset(buffer_index as _) = left_brightness;
                        *right_buffer_ptr.offset(buffer_index as _) = right_brightness;
                    }
                }
            }
        }

        video_frame_sink.append((left_buffer, right_buffer));
    }
}
