
use nom::{le_u8, le_u16, le_u32, le_u64, IResult};
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::io::Cursor;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use super::AttributeType;
use super::RESIDENT_HEADER_SIZE;
#[derive(Debug)]
pub struct FilenameAttr {
    pub parent_id: u64,
    pub allocated_size: u64,
    pub real_size: u64,
    pub flags: u32,
    pub namespace: u8,
    pub name: String,
}
pub fn filename_attr(input: &[u8]) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        take!(4) >>
        attr_offset: le_u16 >>
        skipped_values: value!(attr_offset as u32 - RESIDENT_HEADER_SIZE) >>
        take!(skipped_values) >>
        parent: le_u64 >>
        take!(32) >>
        allocated_size: le_u64 >>
        real_size: le_u64 >>
        flags: le_u32 >>
        take!(4) >>
        name_length: map!(le_u8, |x| x as u16 *2) >>
        namespace: le_u8 >>
        name: take!(name_length) >>
        (AttributeType::Filename(
                FilenameAttr{
                    parent_id: parent,
                    allocated_size: allocated_size,
                    real_size: real_size,
                    namespace: namespace,
                    flags: flags,
                    name: windows_string(name)
        }))
    )
}

fn windows_string(input: &[u8]) -> String {
    let mut x: Vec<u16> = vec![];
    for c in input.chunks(2) {
        let i: u16 = Cursor::new(c).read_u16::<LittleEndian>().unwrap();
        x.push(i);
    }
    OsString::from_wide(&x[..]).into_string().unwrap()
}
