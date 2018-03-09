use std::io::Cursor;
use byteorder::ReadBytesExt;
use byteorder::LittleEndian;
use std::ffi::OsString;
use std::os::windows::prelude::*;

#[derive(Debug, PartialEq)]
pub enum AttributeType {
    Standard(StandardAttr),
    Filename(FilenameAttr),
    Data(Vec<Datarun>),
}

#[derive(Debug)]
pub struct Attribute {
    pub attr_flags: u16,
    pub attr_type: AttributeType,
}

#[derive(Debug, PartialEq)]
pub struct StandardAttr {
    pub dos_flags: u32,
    pub modified: u64,
    pub created: u64,
}

#[derive(Debug, PartialEq)]
pub struct FilenameAttr {
    pub parent_id: u64,
    pub allocated_size: u64,
    pub real_size: u64,
    pub flags: u32,
    pub namespace: u8,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Datarun {
    pub length_lcn: u64,
    pub offset_lcn: i64,
}

pub fn windows_string(input: &[u8]) -> String {
    let mut x: Vec<u16> = vec![];
    for c in input.chunks(2) {
        let i: u16 = Cursor::new(c).read_u16::<LittleEndian>().unwrap();
        x.push(i);
    }
    OsString::from_wide(&x[..]).into_string().unwrap()
}

pub fn length_in_lcn(input: &[u8]) -> u64 {
    let mut base: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    for (i, b) in input.iter().take(8).enumerate() {
        base[i] = *b;
    }
    let mut rdr = Cursor::new(&base);
    rdr.read_u64::<LittleEndian>().unwrap()
}

pub fn offset_in_lcn(input: &[u8]) -> i64 {
    let result = length_in_lcn(input) as i64;
    let last = if input.len() > 7 {
        input.get(7)
    } else {
        input.last()
    };
    if *last.unwrap_or(&0) >= 0x80 {
        -result
    } else {
        result
    }
}
