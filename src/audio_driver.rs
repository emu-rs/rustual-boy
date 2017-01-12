pub trait AudioDriver {
    fn output_frame(&mut self, frame: (i16, i16));
}
