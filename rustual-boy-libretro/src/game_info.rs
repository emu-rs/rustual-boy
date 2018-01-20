extern crate libc;
use self::libc::{c_char, c_void, size_t};

extern crate std;
use std::slice;

#[repr(C)]
pub struct GameInfo {
    path: *const c_char,
    data: *const c_void,
    size: size_t,
    meta: *const c_char,
}

impl GameInfo {
    pub unsafe fn data_ref(&self) -> &[u8] {
        slice::from_raw_parts(self.data as *const u8, self.size)
    }
}
