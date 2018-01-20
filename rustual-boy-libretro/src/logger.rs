extern crate log;

use log::{LogRecord, LogLevel, LogMetadata};

use std::io::{Write, stderr};

struct StderrLogger;

#[allow(unused_must_use)]
impl log::Log for StderrLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            writeln!(&mut stderr(), "[{}] {}", record.level(), record.args());
        }
    }
}

pub fn init() {
    log::set_logger(|max_log_level| {
            max_log_level.set(log::LogLevelFilter::max());
            Box::new(StderrLogger)
        })
        .expect("Unable to register logger");
}
