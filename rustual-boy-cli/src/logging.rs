// When the logging feature is not used, these should effectively do nothing.
//  However, if we simply ignore the arguments passed to the macro, we'll get
//  a bunch of warnings about dead code, unused variables, etc. To get around
//  this, we can bind all of the arguments in a local scope. However, simply
//  binding the arguments will take ownership of them, which is not always
//  desired, so we bind references to them instead. These bindings should
//  be optimized away by the compiler.

#[macro_export]
macro_rules! log {
 ($($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-cli")]
    print!($($arg),*);
 });
}

#[macro_export]
macro_rules! logln {
 ($($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-cli")]
    println!($($arg),*);
 });
}