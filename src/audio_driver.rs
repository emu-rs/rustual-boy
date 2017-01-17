pub trait AudioDriver {
    fn append_frame(&mut self, frame: (i16, i16));
}
