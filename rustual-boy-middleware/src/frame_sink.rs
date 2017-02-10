use rustual_boy_core::sinks::{Sink, VideoFrame, VideoFrameSink};

/// A sink that keeps track of only the most recent
pub struct MostRecentFrameSink<T> {
    inner: Option<T>
}

impl<T> MostRecentFrameSink<T> {
    pub fn new() -> MostRecentFrameSink<T> {
        MostRecentFrameSink { inner: None }
    }

    /// Returns true when we have a frame available
    pub fn has_frame(&self) -> bool {
        self.inner.is_some()
    }

    /// Convert ourself in to a frame
    pub fn into_frame(self) -> Option<T> {
        self.inner
    }
}

impl<T> Sink<T> for MostRecentFrameSink<T> {
    fn append(&mut self, frame: VideoFrame) {
        self.inner = Some(frame);
    }
}

impl VideoFrameSink for MostRecentFrameSink {}
