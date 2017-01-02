#[macro_export]
#[cfg(feature = "logging")]
macro_rules! log {
    ($fmt:expr) => (print!($fmt));
    ($fmt:expr, $($arg:tt)*) => (print!($fmt, $($arg)*));
}

#[macro_export]
#[cfg(feature = "logging")]
macro_rules! logln {
    ($fmt:expr) => (println!($fmt));
    ($fmt:expr, $($arg:tt)*) => (println!($fmt, $($arg)*));
}

#[macro_export]
#[cfg(not(feature = "logging"))]
macro_rules! log {
    ($fmt:expr) => (());
    ($fmt:expr, $($arg:tt)*) => (());
}

#[macro_export]
#[cfg(not(feature = "logging"))]
macro_rules! logln {
    ($fmt:expr) => (());
    ($fmt:expr, $($arg:tt)*) => (());
}
