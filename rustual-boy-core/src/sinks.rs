pub type AudioFrame = (i16, i16);

pub struct AudioSink<'a> {
    pub buffer: &'a mut [AudioFrame],
    pub buffer_pos: usize,
}

pub enum StereoVideoFormat {
    AnaglyphRedElectricCyan,
}

pub enum PixelBuffer<'a> {
    Xrgb1555(&'a mut [u16], usize),
    Rgb565(&'a mut [u16], usize),
    Xrgb8888(&'a mut [u32], usize),
}

impl<'a> PixelBuffer<'a> {
    pub fn pitch(&self) -> usize {
        match self {
            &PixelBuffer::Xrgb1555(_, pitch) => pitch,
            &PixelBuffer::Rgb565(_, pitch) => pitch,
            &PixelBuffer::Xrgb8888(_, pitch) => pitch,
        }
    }
}

pub enum GammaCorrection {
    None,
    TwoPointTwo,
}

pub struct VideoSink<'a> {
    pub buffer: PixelBuffer<'a>,
    pub format: StereoVideoFormat,
    pub gamma_correction: GammaCorrection,
    pub is_populated: bool,
}
