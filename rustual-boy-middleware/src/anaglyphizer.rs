use color::Color;
use color_frame::ColorFrame;
use rustual_boy_core::sinks::{Sink, VideoFrame};
use rustual_boy_core::vip::DISPLAY_PIXELS;

/// A utility for the Rustual Boy core that collapses the left/right
/// anaglyph channels in to a single buffer.
pub struct Anaglyphizer<T: Sink<ColorFrame>> {
    /// Color of the left channel
    left_color: Color,
    /// Color of the right channel
    right_color: Color,
    /// Sink to which we push our frame as they come in
    inner: T
}

impl<T: Sink<ColorFrame>> Anaglyphizer<T> {
    /// Create a new Anaglyphizer which will use the provided colors for
    /// the left and right channels
    pub fn new(inner: T, left_color: Color, right_color: Color) -> Anaglyphizer<T> {
        Anaglyphizer {
            left_color: left_color,
            right_color: right_color,
            inner: inner,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Sink<ColorFrame>> Sink<VideoFrame> for Anaglyphizer<T> {
    fn append(&mut self, frame: VideoFrame) {
        let mut output = Vec::new();
        output.reserve_exact(DISPLAY_PIXELS as usize);
        let (ref l_buffer, ref r_buffer) = frame;

        unsafe {
            let l_buffer = l_buffer.as_ptr();
            let r_buffer = r_buffer.as_ptr();
            {
                let o_ptr = output.as_mut_ptr();
                for i in 0..(DISPLAY_PIXELS as isize) {
                    let l = *(l_buffer.offset(i));
                    let r = *(r_buffer.offset(i));

                    let l = self.left_color.scale_by(l);
                    let r = self.right_color.scale_by(r);

                    *o_ptr.offset(i) = l + r;
                }
            }
            output.set_len(DISPLAY_PIXELS as usize);
        }
        self.inner.append(output.into_boxed_slice());
    }
}
