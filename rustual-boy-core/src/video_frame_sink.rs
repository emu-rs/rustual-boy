pub trait VideoFrameSink {
    fn append(&mut self, frame: (Box<[u8]>, Box<[u8]>));
}
