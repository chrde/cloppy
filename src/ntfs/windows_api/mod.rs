use byteorder::{ByteOrder, LittleEndian};
pub use self::api_calls::*;
pub use self::structs::*;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

mod structs;
mod api_calls;


pub fn windows_string(input: &[u8]) -> String {
    let mut x: Vec<u16> = vec![];
    for c in input.chunks(2) {
        x.push(LittleEndian::read_u16(c));
    }
    OsString::from_wide(&x[..]).into_string().unwrap()
}

