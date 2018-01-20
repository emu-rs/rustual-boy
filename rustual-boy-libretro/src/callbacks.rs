extern crate libc;
use libc::{c_void, size_t};

pub type EnvironmentCallback = extern "C" fn(u32, *mut c_void) -> bool;
pub type VideoRefreshCallback = extern "C" fn(*const c_void, u32, u32, size_t);
pub type AudioSampleCallback = extern "C" fn(i16, i16);
pub type AudioSampleBatchCallback = extern "C" fn(*const i16, size_t);
pub type InputPollCallback = extern "C" fn();
pub type InputStateCallback = extern "C" fn(u32, u32, u32, u32) -> i16;

#[derive(Default)]
pub struct Callbacks {
    pub environment_fn: Option<EnvironmentCallback>,
    pub video_refresh_fn: Option<VideoRefreshCallback>,
    pub audio_sample_fn: Option<AudioSampleCallback>,
    pub audio_sample_batch_fn: Option<AudioSampleBatchCallback>,
    pub input_poll_fn: Option<InputPollCallback>,
    pub input_state_fn: Option<InputStateCallback>,
}

impl Callbacks {
    pub fn environment(&self, cmd: u32, data: *mut c_void) -> bool {
        (self.environment_fn.unwrap())(cmd, data)
    }

    pub fn video_refresh(&self, data: *const c_void, width: u32, height: u32, pitch: size_t) {
        (self.video_refresh_fn.unwrap())(data, width, height, pitch)
    }

    #[allow(dead_code)]
    pub fn audio_sample(&self, left: i16, right: i16) {
        (self.audio_sample_fn.unwrap())(left, right);
    }

    #[allow(dead_code)]
    pub fn audio_sample_batch(&self, data: *const i16, frames: size_t) {
        (self.audio_sample_batch_fn.unwrap())(data, frames);
    }

    pub fn input_poll(&self) {
        (self.input_poll_fn.unwrap())()
    }

    pub fn input_state(&self, port: u32, device: u32, index: u32, id: u32) -> i16 {
        (self.input_state_fn.unwrap())(port, device, index, id)
    }
}
