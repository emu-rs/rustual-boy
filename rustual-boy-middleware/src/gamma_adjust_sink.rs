use color_frame::ColorFrame;
use rustual_boy_core::sinks::Sink;
use rustual_boy_core::vip::DISPLAY_PIXELS;

/// A utility for adjusting a ColorFrame's gamma curve.
/// Typically used with a gamma of 2.2 to prepare a linear
/// buffer for sRGB pixel output.
pub struct GammaAdjustSink<T: Sink<ColorFrame>> {
    inner: T,
    gamma_table: Box<[u8; 256]>,
}

impl<T: Sink<ColorFrame>> GammaAdjustSink<T> {
    /// Create a new GammaAdjustSink which will use the provided gamma
    /// value for adjustment (typically 2.2 for basic RGB -> sRGB
    /// conversion).
    pub fn new(inner: T, gamma: f64) -> GammaAdjustSink<T> {
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

        GammaAdjustSink {
            inner: inner,
            gamma_table: gamma_table,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Sink<ColorFrame>> Sink<ColorFrame> for GammaAdjustSink<T> {
    fn append(&mut self, frame: ColorFrame) {
        let mut output = Vec::new();
        output.reserve_exact(DISPLAY_PIXELS as usize);

        unsafe {
            let input_buffer_ptr = frame.as_ptr();
            {
                let output_buffer_ptr = output.as_mut_ptr();
                for i in 0..(DISPLAY_PIXELS as isize) {
                    let ref input = *(input_buffer_ptr.offset(i));
                    let (input_r, input_g, input_b) = input.into();

                    let output_r = self.gamma_table[input_r as usize];
                    let output_g = self.gamma_table[input_g as usize];
                    let output_b = self.gamma_table[input_b as usize];

                    *output_buffer_ptr.offset(i) = (output_r, output_g, output_b).into();
                }
            }
            output.set_len(DISPLAY_PIXELS as usize);
        }
        self.inner.append(output.into_boxed_slice());
    }
}
