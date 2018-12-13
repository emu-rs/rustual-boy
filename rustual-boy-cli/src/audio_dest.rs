use rustual_boy_core::sinks::AudioFrame;

pub trait AudioDest {
    fn append(&mut self, buffer: &[AudioFrame]);
}
