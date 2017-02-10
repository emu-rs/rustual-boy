extern crate rustual_boy_core;

pub mod color;

pub use color::Color;

use rustual_boy_core::sinks::{Sink, VideoFrame, VideoFrameSink};
use rustual_boy_core::vip::{DISPLAY_RESOLUTION_X, DISPLAY_RESOLUTION_Y};

const DISPLAY_PIXELS: usize = DISPLAY_RESOLUTION_X * DISPLAY_RESOLUTION_Y;

/// A VideoFrameSink for the Rustual Boy core that collapses the left/right
/// anaglyph channels in to a single buffer.
pub struct AnaglyphFrameSink {
    /// Color of the left channel
    left_color: Color,
    /// Color of the right channel
    right_color: Color,

    inner: Option<VideoFrame>,
}

impl AnaglyphFrameSink {
    /// Create a new AnaglyphFrameSink which will use the provided colors for
    /// the left and right channels
    pub fn new(left_color: Color, right_color: Color) -> AnaglyphFrameSink {
        AnaglyphFrameSink {
            left_color: left_color,
            right_color: right_color,
            inner: None
        }
    }

    pub fn update_availible(&self) -> bool {
        self.inner.is_some()
    }

    /// Compute the final anaglyph image, using the most recent frame and
    /// dumping the output in to the provided buffer. If there is no data
    /// availible in the frame buffer, the output argument is untouched
    /// Returns true if a frame update was performed
    pub fn update_output_buffer(&self, output: &mut [u32]) -> bool {
        if output.len() < DISPLAY_PIXELS {
            panic!("Display output buffer not large enough");
        }
        let &(ref l_buffer, ref r_buffer) = match self.inner {
            Some(ref b) => b,
            None => {
                // Nothing to do, since we don't have any data
                return false;
            }
        };

        unsafe {
            let l_buffer = l_buffer.as_ptr();
            let r_buffer = r_buffer.as_ptr();
            let o_ptr = output.as_mut_ptr();
            for i in 0..(DISPLAY_PIXELS as isize) {
                let l = *(l_buffer.offset(i));
                let r = *(r_buffer.offset(i));

                let l = self.left_color.scale_by(l);
                let r = self.right_color.scale_by(r);

                let c = l.add_color(r);

                *o_ptr.offset(i) = c.into();
            }
        }

        true
    }
}

impl Sink<VideoFrame> for AnaglyphFrameSink {
    fn append(&mut self, frame: VideoFrame) {
        self.inner = Some(frame);
    }
}

impl VideoFrameSink for AnaglyphFrameSink {
}
