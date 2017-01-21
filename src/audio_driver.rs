pub trait AudioDriver {
    fn desired_frames(&self) -> usize;
    fn append_frame(&mut self, frame: (i16, i16));
}
