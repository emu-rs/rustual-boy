/// Represents a sink
pub trait Sink<T> {
    /// Push a value out the sink
    fn append(&mut self, value: T);
}

/// Represents a sink of value references.
pub trait SinkRef<T: ?Sized> {
    fn append(&mut self, value: &T);
}

/// A frame of video. The Boxs contain the left/right monochrome
/// [DISPLAY_RESOLUTION_X](../vip/constant.DISPLAY_RESOLUTION_X.html) by
/// [DISPLAY_RESOLUTION_Y](../vip/constant.DISPLAY_RESOLUTION_Y.html)
/// pixels, after gamma mapping.
pub type VideoFrame = (Box<[u8]>, Box<[u8]>);

/// A sink for video frames.
pub trait VideoFrameSink: Sink<VideoFrame> {}

/// A sink for audio buffers
pub trait AudioBufferSink: SinkRef<[(i16, i16)]> {}

/// A sink for individual audio frames
pub trait AudioFrameSink: Sink<(i16, i16)> {}
