pub trait VideoDriver {
    fn output_frame(&mut self, frame: &[u32]);
}
