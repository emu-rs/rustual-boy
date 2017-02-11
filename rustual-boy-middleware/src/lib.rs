extern crate rustual_boy_core;

mod color;
mod color_frame;
mod anaglyphizer;
mod gamma_adjust_sink;
mod most_recent_sink;

// reexports
pub use color::Color;
pub use anaglyphizer::Anaglyphizer;
pub use gamma_adjust_sink::GammaAdjustSink;
pub use most_recent_sink::MostRecentSink;
