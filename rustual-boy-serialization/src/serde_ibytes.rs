use std::{mem, slice};
use serde_bytes::ByteBuf;
use serde::{Serializer, Deserializer, Deserialize};

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: From<Vec<i8>>,
          D: Deserializer<'de>
{
    ByteBuf::deserialize(deserializer).map(|buf| {
        let mut u8_v: Vec<u8> = buf.into();

        let i8_v = unsafe {
            Vec::from_raw_parts(
                u8_v.as_mut_ptr() as *mut i8,
                u8_v.len(),
                u8_v.capacity()
            )
        };
        mem::forget(u8_v);
        i8_v.into()
    })
}

pub fn serialize<T, S>(bytes: &T, serializer: S) -> Result<S::Ok, S::Error>
    where T: ?Sized + AsRef<[i8]>,
          S: Serializer
{
    let i8_s: &[i8] = bytes.as_ref();
    let u8_s: &[u8] = unsafe { slice::from_raw_parts(i8_s.as_ptr() as *const u8, i8_s.len()) };
    serializer.serialize_bytes(u8_s)
}
