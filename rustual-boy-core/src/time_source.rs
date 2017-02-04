pub trait TimeSource {
    fn time_ns(&self) -> u64;
}
