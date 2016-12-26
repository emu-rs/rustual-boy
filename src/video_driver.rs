pub trait VideoDriver {
    fn output_frame(&mut self, frame: Box<[u32]>);
}
