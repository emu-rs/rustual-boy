pub trait VideoDriver {
    fn output_frame(&mut self, frame: (Box<[u8]>, Box<[u8]>));
}
