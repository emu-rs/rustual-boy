extern crate libc;
use libc::c_void;

use std::mem::size_of;

extern crate rustual_boy_core;
use rustual_boy_core::sinks::{Sink, AudioFrame};

use rustual_boy_core::vip::{DISPLAY_RESOLUTION_X, DISPLAY_RESOLUTION_Y, DISPLAY_PIXELS};

extern crate rustual_boy_middleware;
use rustual_boy_middleware::ColorFrame;

use ::callbacks::Callbacks;

pub struct CallbackSink(pub &'static Callbacks);

impl Sink<ColorFrame> for CallbackSink {
    fn append(&mut self, frame: ColorFrame) {
        let callbacks = self.0;

        let output_bytes_per_pixel = size_of::<u32>();
        let output_size_bytes = (DISPLAY_PIXELS as usize) * output_bytes_per_pixel;

        let mut output: Vec<u32> = Vec::new();
        output.reserve_exact(output_size_bytes);

        unsafe {
            let input_ptr = frame.as_ptr();
            {
                let output_ptr = output.as_mut_ptr();
                for i in 0..(DISPLAY_PIXELS as isize) {
                    let ref input_color = *(input_ptr.offset(i));

                    *output_ptr.offset(i) = input_color.into();
                }
            }
            output.set_len(output_size_bytes);
        }

        let output_ptr = Box::into_raw(output.into_boxed_slice());

        callbacks.video_refresh(output_ptr as *mut c_void,
                                DISPLAY_RESOLUTION_X,
                                DISPLAY_RESOLUTION_Y,
                                (DISPLAY_RESOLUTION_X as usize) * output_bytes_per_pixel);

        unsafe {
            Box::from_raw(output_ptr);
        }
    }
}

impl Sink<AudioFrame> for CallbackSink {
    fn append(&mut self, frame: AudioFrame) {
        let callbacks = self.0;

        let (left, right) = frame;
        callbacks.audio_sample(left, right);
    }
}
