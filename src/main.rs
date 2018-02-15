#[macro_use]
extern crate nom;
extern crate byteorder;

use std::io::Cursor;
use nom::IResult;
use nom::{le_u8, le_u16, le_u32, le_u64};
use byteorder::{LittleEndian, ReadBytesExt};
use std::ffi::OsString;
use std::os::windows::prelude::*;

#[derive(Debug)]
enum AttributeType {
    Standard(StandardAttr),
    Filename(FilenameAttr),
    Data(Vec<Datarun>),
    End,
    Ignored,
}

#[derive(Default, Debug)]
struct FileEntry {
    flags: u16,
    id: u32,

    dos_flags: u32,
    parent_id: u64,
    real_size: u64,
    logical_size: u64,
}

#[derive(Debug)]
struct StandardAttr {
    dos_flags: u32,
    modified: u64,
    created: u64,
}

#[derive(Debug)]
struct FilenameAttr {
    parent_id: u64,
    allocated_size: u64,
    real_size: u64,
    flags: u32,
    namespace: u8,
    name: String,
}

struct FixupBytes<'a> {
    attr_offset: u16,
    fixup_seq: &'a [u8],

}

#[derive(Debug)]
struct Attribute {
    attr_flags: u16,
    attr_type: AttributeType,
}

#[derive(Debug)]
struct Datarun {
    length_lcn: u64,
    offset_lcn: i64,
}



const ATTR_HEADER_SIZE: u32 = 0x10;
const STANDARD_ATTR_SIZE: u32 = 0x24;
//attr_length and attr_offset
const RESIDENT_HEADER_SIZE: u32 = ATTR_HEADER_SIZE + 4 + 2;
//starting_vcn, last_vcn and datarun_offset
const NONRESIDENT_HEADER_SIZE: u32 = ATTR_HEADER_SIZE + 8 + 8 + 2;
const END: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
const DATARUN_END: [u8; 1] = [0x00];

fn main() {
    let fr0: &[u8; 1024] = &[70, 73, 76, 69, 48, 0, 3, 0, 80, 245, 122, 254, 24, 0, 0, 0, 1, 0, 1, 0, 56, 0, 1, 0, 184, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 96, 6, 255, 255, 0, 0, 0, 0, 16, 0, 0, 0, 96, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 72, 0, 0, 0, 24, 0, 0, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 0, 0, 0, 104, 0, 0, 0, 0, 0, 24, 0, 0, 0, 3, 0, 74, 0, 0, 0, 24, 0, 1, 0, 5, 0, 0, 0, 0, 0, 5, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 4, 3, 36, 0, 77, 0, 70, 0, 84, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 104, 0, 0, 0, 1, 0, 64, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 83, 8, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 51, 32, 200, 0, 0, 0, 12, 67, 236, 207, 0, 118, 65, 153, 0, 67, 237, 201, 0, 94, 217, 243, 0, 51, 72, 235, 0, 12, 153, 121, 67, 191, 6, 5, 60, 11, 224, 0, 0, 0, 176, 0, 0, 0, 72, 0, 0, 0, 1, 0, 64, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 66, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 49, 67, 118, 24, 3, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 94, 177, 15, 1, 224, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
//    println!("{}", buffer.0.to_hex(16));
    fixup_buffer(fr0);
}

fn fixup_buffer(buffer: &[u8]) {
    let bytesPerSector = 512;
    let bytesPerFR = 1024;
    let sectorsPerFR = bytesPerFR / bytesPerSector;
    let res = file_record_header(buffer);
    match res {
        IResult::Done(i, r) => {
            println!("parsed: {:?} {:?}", r.0, &r.1.fixup_seq[2..]);
            let x: &[u8] = r.1.fixup_seq;
            for (i, chunk) in x.chunks(2).skip(1).enumerate() {
                println!("updating position {} with value {}", bytesPerSector * (i + 1) - 2, chunk.first().unwrap());
                println!("updating position {} with value {}", bytesPerSector * (i + 1) - 1, chunk.last().unwrap());
            }
            match all_attributes(&buffer[r.1.attr_offset as usize..]) {
                IResult::Done(i, r) => {
                    println!("{:?}", r);
                }
                _ => {
                    println!("error or incomplete");
                    panic!("cannot parse header");
                }
            }
        }
        _ => {
            println!("error or incomplete");
            panic!("cannot parse header");
        }
    }
}

fn file_record_header(input: &[u8]) -> IResult<&[u8], (FileEntry, FixupBytes)> {
    do_parse!(input,
        tag!(b"FILE") >>
        take!(2) >>
        fixup_size: le_u16 >>
        take!(12) >>
        attr_offset: le_u16 >>
        flags: le_u16 >>
        take!(20) >>
        fr_number: le_u32 >>
        fixup_seq: take!(2 * fixup_size) >>
        (FileEntry{flags: flags, id: fr_number, ..Default::default()}, FixupBytes{attr_offset: attr_offset, fixup_seq: fixup_seq})
    )
}

fn attributes(input: &[u8]) -> IResult<&[u8], Attribute> {
    do_parse!(input,
        current_pos: curr_position >>
        attr_type: le_u32 >>
        attr_length: le_u32 >>
        resident: le_u8 >>
        name_length: le_u8 >>
        name_offset: le_u16 >>
        flags: le_u16 >>
        id: le_u16 >>
        attr: switch!(value!(attr_type),
                    0x10 => call!(standard_attr) |
                    0x30 => call!(filename_attr) |
                    0x80 => call!(data_attr) |
                    _ => value!(AttributeType::Ignored)) >>
        new_pos: curr_position >>
        take!(attr_length - (current_pos - new_pos)) >>
        (Attribute{
            attr_flags: flags,
            attr_type: attr,
        })
    )
}

fn data_attr(input: &[u8]) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        take!(16) >>
        datarun_offset: le_u16 >>
        skipped_values: value!(datarun_offset as u32 - NONRESIDENT_HEADER_SIZE) >>
        take!(skipped_values) >>
        data: dataruns >>
        (AttributeType::Data(data))
    )
}

fn curr_position(input: &[u8]) -> IResult<&[u8], u32> {
    IResult::Done(input, input.len() as u32)
}

fn datarun(input: &[u8]) -> IResult<&[u8], Datarun> {
    do_parse!(input,
        header: split_datarun_header >>
        length_lcn: map!(take!(header.1), length_in_lcn) >>
        offset_lcn: map!(take!(header.0), offset_in_lcn) >>
        (Datarun{length_lcn: length_lcn, offset_lcn: offset_lcn})
    )
}

fn dataruns(input: &[u8]) -> IResult<&[u8], Vec<Datarun> > {
    do_parse!(input,
        data: many_till!(call!(datarun), tag!(DATARUN_END)) >>
            (data.0)
    )
}

named!(split_datarun_header<(u8, u8)>, bits!( pair!( take_bits!(u8, 4), take_bits!(u8, 4) ) ) );

fn length_in_lcn(input: &[u8]) -> u64 {
    let mut base: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    for (i, b) in input.iter().take(8).enumerate() {
        base[i] = *b;

    }
    let mut rdr = Cursor::new(&base);
    rdr.read_u64::<LittleEndian>().unwrap()
}

fn offset_in_lcn(input: &[u8]) -> i64 {
    let result = length_in_lcn(input) as i64;
    if *input.last().unwrap_or(&0) >= 0x80 {
        -result
    } else{
        result
    }
}

fn filename_attr(input: &[u8]) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        attr_length: le_u32 >>
        attr_offset: le_u16 >>
        skipped_values: value!(attr_offset as u32 - RESIDENT_HEADER_SIZE) >>
        take!(skipped_values) >>
        parent: le_u64 >>
        take!(32) >>
        allocated_size: le_u64 >>
        real_size: le_u64 >>
        flags: le_u32 >>
        take!(4) >>
        name_length: map!(le_u8, |x| x*2) >>
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

fn standard_attr(input: &[u8]) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        attr_length: le_u32 >>
        attr_offset: le_u16 >>
        skipped_values: value!(attr_offset as u32 - RESIDENT_HEADER_SIZE) >>
        take!(skipped_values) >>
        created: le_u64 >>
        modified: le_u64 >>
        take!(16) >>
        dos_flags: le_u32 >>
        (AttributeType::Standard(
                StandardAttr{
                    modified: modified,
                    created: created,
                    dos_flags: dos_flags
                }))
    )
}

fn all_attributes(input: &[u8]) -> IResult<&[u8], (Vec<Attribute>, &[u8])> {
    many_till!(input, call!(attributes), tag!(END))
}

fn windows_string(input: &[u8]) -> String {
    let mut x: Vec<u16> = vec![];
    for c in input.chunks(2) {
        let i: u16 = std::io::Cursor::new(c).read_u16::<LittleEndian>().unwrap();
        x.push(i);
    }
    OsString::from_wide(&x[..]).into_string().unwrap()
}