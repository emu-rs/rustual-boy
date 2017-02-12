// This enum is only referenced in macros and macro definations.  When macros
//  get resolved it will remove the references to the enum in the generated
//  code.  This has the effect of making the enum unused from the compiler's
//  point of view.
#[allow(dead_code)]
pub enum Log {
  Cpu,
  GamePad,
  Ic,
  Vip,
  Vsu,
}

// When the logging features aren't used, these should effectively do nothing.
//  However, if we simply ignore the arguments passed to the macro, we'll get
//  a bunch of warnings about dead code, unused variables, etc. To get around
//  this, we can bind all of the arguments in a local scope. However, simply
//  binding the arguments will take ownership of them, which is not always
//  desired, so we bind references to them instead. These bindings should
//  be optimized away by the compiler.
#[macro_export]
macro_rules! log {
 (Log::Cpu, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-cpu")]
    print!($($arg),*);
 });
 (Log::GamePad, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-gamepad")]
    print!($($arg),*);
 });
 (Log::Ic, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-ic")]
    print!($($arg),*);
 });
 (Log::Vip, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-vip")]
    print!($($arg),*);
 });
 (Log::Vsu, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-vsu")]
    print!($($arg),*);
 });
 ($($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-other")]
    print!($($arg),*);
 });
}

#[macro_export]
macro_rules! logln {
 (Log::Cpu, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-cpu")]
    println!($($arg),*);
 });
 (Log::GamePad, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-gamepad")]
    println!($($arg),*);
 });
 (Log::Ic, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-ic")]
    println!($($arg),*);
 });
 (Log::Vip, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-vip")]
    println!($($arg),*);
 });
 (Log::Vsu, $($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-vsu")]
    println!($($arg),*);
 });
 ($($arg:expr),*) => ({
    $(let _arg = &$arg;)*
    #[cfg(feature = "log-other")]
    println!($($arg),*);
 });
}