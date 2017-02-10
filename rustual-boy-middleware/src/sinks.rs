use rustual_boy_core::sinks::{Sink, VideoFrame};

/// A sink that keeps track of only the most recent value
pub struct MostRecentSink<T> {
    inner: Option<T>
}

impl<T> MostRecentSink<T> {
    pub fn new() -> MostRecentSink<T> {
        MostRecentSink { inner: None }
    }

    /// Returns true when we have a frame available
    pub fn has_frame(&self) -> bool {
        self.inner.is_some()
    }

    /// Convert ourself in to a frame
    pub fn into_inner(self) -> Option<T> {
        self.inner
    }
}

impl<T> Sink<T> for MostRecentSink<T> {
    fn append(&mut self, v: T) {
        self.inner = Some(v);
    }
}
