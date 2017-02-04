// This pass-through version should use $arg:tt rather than $arg:expr, but
//  I've changed it to match the definition below, which needs the separate
//  args, not just tokens. Otherwise, if there are any subtle differences
//  between the tt/expr semantics, we may end up in a situation where the
//  code compiles in one configuration, but not the other. So far, there
//  haven't been any noticeable differences in this codebase. I'm guessing
//  tt is used in the original println just for simplicity.

#[macro_export]
#[cfg(feature = "logging")]
macro_rules! log {
    ($fmt:expr) => (print!($fmt));
    ($fmt:expr, $($arg:expr),*) => (print!($fmt, $($arg),*));
}

#[macro_export]
#[cfg(feature = "logging")]
macro_rules! logln {
    ($fmt:expr) => (println!($fmt));
    ($fmt:expr, $($arg:expr),*) => (println!($fmt, $($arg),*));
}

// When the logging feature is not used, these should effectively do nothing.
//  However, if we simply ignore the arguments passed to the macro, we'll get
//  a bunch of warnings about dead code, unused variables, etc. To get around
//  this, we can bind all of the arguments in a local scope. However, simply
//  binding the arguments will take ownership of them, which is not always
//  desired, so we bind references to them instead. These bindings should
//  be optimized away by the compiler.

#[macro_export]
#[cfg(not(feature = "logging"))]
macro_rules! log {
    ($fmt:expr) => (());
    ($fmt:expr, $($arg:expr),*) => ({
        $(
            let _arg = &$arg;
        )*
    });
}

#[macro_export]
#[cfg(not(feature = "logging"))]
macro_rules! logln {
    ($fmt:expr) => (());
    ($fmt:expr, $($arg:expr),*) => ({
        $(
            let _arg = &$arg;
        )*
    });
}
