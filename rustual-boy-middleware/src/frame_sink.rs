use rustual_boy_core::sinks::{Sink, VideoFrame, VideoFrameSink};

/// A frame sink that keeps track of only the most recent frame
pub struct MostRecentFrameSink {
    inner: Option<VideoFrame>
}

impl MostRecentFrameSink {
    pub fn new() -> MostRecentFrameSink {
        MostRecentFrameSink { inner: None }
    }

    /// Returns true when we have a frame available
    pub fn has_frame(&self) -> bool {
        self.inner.is_some()
    }

    /// Convert ourself in to a frame
    pub fn into_frame(self) -> Option<VideoFrame> {
        self.inner
    }
}

impl Sink<VideoFrame> for MostRecentFrameSink {
    fn append(&mut self, frame: VideoFrame) {
        self.inner = Some(frame);
    }
}

impl VideoFrameSink for MostRecentFrameSink {}
