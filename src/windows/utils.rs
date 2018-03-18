use std::io;
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use winapi::um::winuser::CW_USEDEFAULT;
use byteorder::{LittleEndian, ByteOrder};

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}

impl<T> ToWide for T where T: AsRef<OsStr> {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}

pub trait FromWide {
    fn from_wide_null(wide: &[u16]) -> Self;
}

impl FromWide for OsString {
    fn from_wide_null(wide: &[u16]) -> OsString {
        let len = wide.iter().take_while(|&&c| c != 0).count();
        OsString::from_wide(&wide[..len])
    }
}

pub fn last_error<T>() -> io::Result<T> {
    Err(io::Error::last_os_error())
}

pub fn other_error<T>(msg: &str) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::Other, msg))
}

pub struct Location {
    pub x: i32,
    pub y: i32,
}

impl Default for Location {
    fn default() -> Self {
        Location { x: CW_USEDEFAULT, y: CW_USEDEFAULT }
    }
}

pub fn windows_string(input: &[u8]) -> String {
    let mut x: Vec<u16> = vec![];
    for c in input.chunks(2) {
        x.push(LittleEndian::read_u16(c));
    }
    OsString::from_wide(&x[..]).into_string().unwrap()
}

