pub trait VideoFrameSink {
    fn append_frame(&mut self, frame: (Box<[u8]>, Box<[u8]>));
}
