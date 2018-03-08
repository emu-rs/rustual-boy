extern crate libc;

extern crate rustual_boy_core;

mod callbacks;
mod game_info;
mod input;
mod retro;
mod system_av_info;
mod system_info;

use libc::*;

use rustual_boy_core::game_pad::*;
use rustual_boy_core::rom::*;
use rustual_boy_core::sinks::*;
use rustual_boy_core::sram::*;
use rustual_boy_core::vip::*;
use rustual_boy_core::vip::mem_map::*;
use rustual_boy_core::virtual_boy::*;
use rustual_boy_core::vsu::*;
use rustual_boy_core::wram::*;

use callbacks::*;
use game_info::*;
use input::*;
use retro::*;
use system_av_info::*;
use system_info::*;

use std::{mem, ptr};
use std::slice;

pub enum OutputBuffer {
    Xrgb1555(Vec<u16>),
    Rgb565(Vec<u16>),
    Xrgb8888(Vec<u32>),
}

struct System {
    virtual_boy: VirtualBoy,
    emulated_cycles: u64,
    target_emulated_cycles: u64,
}

impl System {
    fn new(rom: Rom, sram: Sram) -> System {
        System {
            virtual_boy: VirtualBoy::new(rom, sram),
            emulated_cycles: 0,
            target_emulated_cycles: 0,
        }
    }
}

pub struct Context {
    system: Option<System>,
    video_output_frame_buffer: OutputBuffer,
    audio_frame_buffer: Vec<AudioFrame>,
}

impl Context {
    fn new() -> Context {
        Context {
            system: None,
            video_output_frame_buffer: OutputBuffer::Xrgb1555(vec![0; DISPLAY_PIXELS as usize]),
            audio_frame_buffer: vec![(0, 0); (SAMPLE_RATE as usize) / 50 * 2], // double space needed for 1 frame for lots of skid room
        }
    }

    fn load_game(&mut self, game_info: &GameInfo) -> bool {
        unsafe {
            // It seems retroarch (and possibly other frontends) is a bit finicky with accepting the set pixel format
            //  callback, so we should call it a few times before giving up. We don't want to loop forever here though,
            //  as it's possible that a given frontend just doesn't support changing the pixel format. In that case, we
            //  will want to fall back to using the default pixel format.
            let desired_pixel_format = PixelFormat::Rgb565; // Recommended libretro pixel format
            let mut actual_pixel_format = PixelFormat::Xrgb1555; // Default frontend pixel format
            for _ in 0..10 {
                if CALLBACKS.set_pixel_format(desired_pixel_format) {
                    actual_pixel_format = desired_pixel_format;
                    break;
                }
            }

            self.video_output_frame_buffer = match actual_pixel_format {
                PixelFormat::Xrgb1555 => OutputBuffer::Xrgb1555(vec![0; DISPLAY_PIXELS as usize]),
                PixelFormat::Rgb565 => OutputBuffer::Rgb565(vec![0; DISPLAY_PIXELS as usize]),
                PixelFormat::Xrgb8888 => OutputBuffer::Xrgb8888(vec![0; DISPLAY_PIXELS as usize]),
            };

            match Rom::from_bytes(game_info.data_ref()) {
                Ok(rom) => {
                    self.system = Some(System::new(rom, Sram::new()));

                    true
                }
                Err(_) => false,
            }
        }
    }

    fn unload_game(&mut self) {
        self.system = None;
    }

    fn system_av_info(&self) -> SystemAvInfo {
        SystemAvInfo {
            geometry: SystemGameGeometry {
                base_width: DISPLAY_RESOLUTION_X,
                base_height: DISPLAY_RESOLUTION_Y,
                max_width: DISPLAY_RESOLUTION_X,
                max_height: DISPLAY_RESOLUTION_Y,

                // Optional
                aspect_ratio: 0.0,
            },
            timing: SystemTiming {
                fps: 50.0,
                sample_rate: SAMPLE_RATE as f64,
            },
        }
    }

    fn reset(&mut self) {
        // Pull out rom/sram from existing system, and build new system around them
        let (rom, sram) = self.system.as_ref().map(|system| (system.virtual_boy.interconnect.rom.clone(), system.virtual_boy.interconnect.sram.clone())).unwrap();
        self.system = Some(System::new(rom, sram));
    }

    fn run_frame(&mut self) {
        unsafe {
            CALLBACKS.input_poll();

            if let Some(ref mut system) = self.system {
                {
                    let game_pad = &mut system.virtual_boy.interconnect.game_pad;

                    game_pad.set_button_pressed(Button::A, CALLBACKS.joypad_button(JoypadButton::A));
                    game_pad.set_button_pressed(Button::B, CALLBACKS.joypad_button(JoypadButton::B));
                    game_pad.set_button_pressed(Button::L, CALLBACKS.joypad_button(JoypadButton::L));
                    game_pad.set_button_pressed(Button::R, CALLBACKS.joypad_button(JoypadButton::R));
                    game_pad.set_button_pressed(Button::Start, CALLBACKS.joypad_button(JoypadButton::Start));
                    game_pad.set_button_pressed(Button::Select, CALLBACKS.joypad_button(JoypadButton::Select));

                    let joypad_left_pressed = CALLBACKS.joypad_button(JoypadButton::Left);
                    let joypad_right_pressed = CALLBACKS.joypad_button(JoypadButton::Right);
                    let joypad_up_pressed = CALLBACKS.joypad_button(JoypadButton::Up);
                    let joypad_down_pressed = CALLBACKS.joypad_button(JoypadButton::Down);

                    const ANALOG_THRESHOLD: i16 = 0x7fff / 2;

                    let (left_x, left_y) = CALLBACKS.analog_xy(AnalogStick::Left);
                    game_pad.set_button_pressed(Button::LeftDPadLeft, left_x < -ANALOG_THRESHOLD || joypad_left_pressed);
                    game_pad.set_button_pressed(Button::LeftDPadRight, left_x > ANALOG_THRESHOLD || joypad_right_pressed);
                    game_pad.set_button_pressed(Button::LeftDPadUp, left_y < -ANALOG_THRESHOLD || joypad_up_pressed);
                    game_pad.set_button_pressed(Button::LeftDPadDown, left_y > ANALOG_THRESHOLD || joypad_down_pressed);

                    let (right_x, right_y) = CALLBACKS.analog_xy(AnalogStick::Right);
                    game_pad.set_button_pressed(Button::RightDPadLeft, right_x < -ANALOG_THRESHOLD);
                    game_pad.set_button_pressed(Button::RightDPadRight, right_x > ANALOG_THRESHOLD);
                    game_pad.set_button_pressed(Button::RightDPadUp, right_y < -ANALOG_THRESHOLD);
                    game_pad.set_button_pressed(Button::RightDPadDown, right_y > ANALOG_THRESHOLD);
                }

                let (pixel_buffer_ptr, pixel_buffer) = match CALLBACKS.get_current_software_framebuffer(DISPLAY_RESOLUTION_X, DISPLAY_RESOLUTION_Y) {
                    Some(fb) => match fb.format {
                        0 => (fb.data, PixelBuffer::Xrgb1555(slice::from_raw_parts_mut(fb.data as *mut _, (fb.height as usize) * (fb.pitch as usize) / mem::size_of::<u16>()), fb.pitch)),
                        1 => (fb.data, PixelBuffer::Xrgb8888(slice::from_raw_parts_mut(fb.data as *mut _, (fb.height as usize) * (fb.pitch as usize) / mem::size_of::<u32>()), fb.pitch)),
                        2 => (fb.data, PixelBuffer::Rgb565(slice::from_raw_parts_mut(fb.data as *mut _, (fb.height as usize) * (fb.pitch as usize) / mem::size_of::<u16>()), fb.pitch)),
                        _ => panic!("Host returned framebuffer with unrecognized pixel format format")
                    }
                    _ => match self.video_output_frame_buffer {
                        OutputBuffer::Xrgb1555(ref mut buffer) =>
                            (buffer.as_mut_ptr() as *mut c_void, PixelBuffer::Xrgb1555(buffer, (DISPLAY_RESOLUTION_X as usize) * mem::size_of::<u16>())),
                        OutputBuffer::Xrgb8888(ref mut buffer) =>
                            (buffer.as_mut_ptr() as *mut c_void, PixelBuffer::Xrgb8888(buffer, (DISPLAY_RESOLUTION_X as usize) * mem::size_of::<u32>())),
                        OutputBuffer::Rgb565(ref mut buffer) =>
                            (buffer.as_mut_ptr() as *mut c_void, PixelBuffer::Rgb565(buffer, (DISPLAY_RESOLUTION_X as usize) * mem::size_of::<u16>())),
                    }
                };

                let mut video_output_sink = VideoSink {
                    buffer: pixel_buffer,
                    format: StereoVideoFormat::AnaglyphRedElectricCyan,
                    gamma_correction: GammaCorrection::TwoPointTwo,
                    is_populated: false,
                };

                // TODO: Remove scope when NLL is stable
                let rendered_audio_frames = {
                    let mut audio_output_sink = AudioSink {
                        buffer: &mut self.audio_frame_buffer,
                        buffer_pos: 0,
                    };

                    system.target_emulated_cycles += 1_000_000_000 / 50 / 50; // 1s period in ns, 50 frames per second, 50ns per cycle

                    while system.emulated_cycles < system.target_emulated_cycles {
                        let (num_cycles, _) = system.virtual_boy.step(&mut video_output_sink, &mut audio_output_sink);
                        system.emulated_cycles += num_cycles as _;
                    }

                    audio_output_sink.buffer_pos
                };

                (CALLBACKS.video_refresh.unwrap())(pixel_buffer_ptr, DISPLAY_RESOLUTION_X, DISPLAY_RESOLUTION_Y, video_output_sink.buffer.pitch());
                (CALLBACKS.audio_sample_batch.unwrap())(self.audio_frame_buffer.as_mut_ptr() as *mut _, rendered_audio_frames as _);
            }
        }
    }
}

static mut CALLBACKS: Callbacks = Callbacks {
    video_refresh: None,
    audio_sample: None,
    audio_sample_batch: None,
    input_poll: None,
    input_state: None,
    environment: None,
};

static mut CONTEXT: *mut Context = 0 as *mut _;

#[no_mangle]
pub extern "C" fn retro_api_version() -> u32 {
    API_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn retro_init() {
    CONTEXT = Box::into_raw(Box::new(Context::new()));
}

#[no_mangle]
pub unsafe extern "C" fn retro_deinit() {
    Box::from_raw(CONTEXT); // Take ownership of CONTEXT and drop it
    CONTEXT = ptr::null_mut();
}

// These `retro_set` fn's can be called _before_ retro_init, so they can't touch any context state

#[no_mangle]
pub unsafe extern "C" fn retro_set_video_refresh(callback: VideoRefreshCallback) {
    CALLBACKS.video_refresh = Some(callback);
}

#[no_mangle]
pub unsafe extern "C" fn retro_set_audio_sample(callback: AudioSampleCallback) {
    CALLBACKS.audio_sample = Some(callback);
}

#[no_mangle]
pub unsafe extern "C" fn retro_set_audio_sample_batch(callback: AudioSampleBatchCallback) {
    CALLBACKS.audio_sample_batch = Some(callback);
}

#[no_mangle]
pub unsafe extern "C" fn retro_set_input_poll(callback: InputPollCallback) {
    CALLBACKS.input_poll = Some(callback);
}

#[no_mangle]
pub unsafe extern "C" fn retro_set_input_state(callback: InputStateCallback) {
    CALLBACKS.input_state = Some(callback);
}

#[no_mangle]
pub unsafe extern "C" fn retro_set_environment(callback: EnvironmentCallback) {
    CALLBACKS.environment = Some(callback);
}

#[no_mangle]
pub unsafe extern "C" fn retro_set_controller_port_device(_port: u32, _device: u32) {
    // TODO
}

#[no_mangle]
pub unsafe extern "C" fn retro_get_system_info(info: *mut SystemInfo) {
    // This can be called _before_ retro_init, so this can't be part of the context
    *info = SystemInfo::new();
}

#[no_mangle]
pub unsafe extern "C" fn retro_load_game(game_info: *const GameInfo) -> bool {
    (*CONTEXT).load_game(&*game_info)
}

#[no_mangle]
pub unsafe extern "C" fn retro_load_game_special(_game_type: u32, _game_infos: *const GameInfo, _num_game_infos: size_t) -> bool {
    // Neither required nor recommended
    false
}

#[no_mangle]
pub unsafe extern "C" fn retro_get_system_av_info(av_info: *mut SystemAvInfo) {
    *av_info = (*CONTEXT).system_av_info();
}

#[no_mangle]
pub unsafe extern "C" fn retro_unload_game() {
    (*CONTEXT).unload_game();
}

#[no_mangle]
pub unsafe extern "C" fn retro_reset() {
    (*CONTEXT).reset();
}

#[no_mangle]
pub unsafe extern "C" fn retro_run() {
    (*CONTEXT).run_frame();
}

#[no_mangle]
pub unsafe extern "C" fn retro_get_region() -> u32 {
    1 // TODO
}

#[no_mangle]
pub unsafe extern "C" fn retro_get_memory_data(id: u32) -> *mut c_void {
    match id & MEMORY_MASK {
        MEMORY_SAVE_RAM => (*CONTEXT).system.as_mut().unwrap().virtual_boy.interconnect.sram.bytes_ptr() as *mut _,
        MEMORY_SYSTEM_RAM => (*CONTEXT).system.as_mut().unwrap().virtual_boy.interconnect.wram.bytes_ptr() as *mut _,
        MEMORY_VIDEO_RAM => (*CONTEXT).system.as_mut().unwrap().virtual_boy.interconnect.vip.vram_ptr() as *mut _,
        _ => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn retro_get_memory_size(id: u32) -> size_t {
    match id & MEMORY_MASK {
        MEMORY_SAVE_RAM => MAX_SRAM_SIZE as _,
        MEMORY_SYSTEM_RAM => WRAM_SIZE as _,
        MEMORY_VIDEO_RAM => VRAM_LENGTH as _,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn retro_serialize_size() -> size_t {
    0
}

#[no_mangle]
pub unsafe extern "C" fn retro_serialize(_data: *mut c_void, _size: size_t) -> bool {
    unimplemented!("retro_serialize");
}

#[no_mangle]
pub unsafe extern "C" fn retro_unserialize(_data: *const c_void, _size: size_t) -> bool {
    unimplemented!("retro_unserialize");
}

#[no_mangle]
pub unsafe extern "C" fn retro_cheat_reset() {
    unimplemented!("retro_cheat_reset");
}

#[no_mangle]
pub unsafe extern "C" fn retro_cheat_set(_index: u32, _enabled: bool, _code: *const c_char) {
    unimplemented!("retro_cheat_set");
}
