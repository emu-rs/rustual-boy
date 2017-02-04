pub trait AudioFrameSink {
    fn append(&mut self, frame: (i16, i16));
}
