use gui::utils;
use std::io;
use winapi::shared::windef::HACCEL;
use winapi::um::winuser::*;
pub const ID_SELECT_ALL: u16 = 0x8000;
pub const ID_FILL_LIST: u16 = 0x8001;

type Entry = (u8, u16, u16);

const ENTRIES: &'static [Entry] = &[
    ((FCONTROL | FVIRTKEY), 0x41, ID_SELECT_ALL),
    ((FCONTROL | FVIRTKEY), 0x42, ID_FILL_LIST),
];

pub fn new() -> io::Result<HACCEL> {
    let mut vec = ENTRIES.iter().map(|entry| {
        ACCEL {
            fVirt: entry.0,
            key: entry.1,
            cmd: entry.2,
        }
    }).collect::<Vec<_>>();
    let size = vec.len() as i32;
    match unsafe {
        CreateAcceleratorTableW(vec.as_mut_ptr(), size)
    } {
        v if v.is_null() => utils::other_error("CreateAcceleratorTableW failed"),
        v => Ok(v)
    }
}