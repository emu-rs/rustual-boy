pub trait AudioBufferSink {
    fn append(&mut self, buffer: &[(i16, i16)]);
}
