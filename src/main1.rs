#[macro_use]
extern crate nom;
extern crate byteorder;

use std::io::Cursor;
use nom::{IResult, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};


pub fn main() {
    println!("{:?}", dataruns(&[0x11, 0x30, 0x20, 0x01, 0x60, 0x11, 0x10, 0x30, 0x00,]));
    println!("{:?}", dataruns(&[0x31, 0x38, 0x73, 0x25, 0x34, 0x32, 0x14, 0x01, 0xE5, 0x11, 0x02, 0x31, 0x42, 0xAA, 0x00, 0x03, 0x00]));
    println!("{:?}", dataruns(&[0x11, 0x30, 0x60, 0x21, 0x10, 0x00, 0x01, 0x11, 0x20, 0xE0, 0x00]));
}
#[derive(Debug)]
struct Datarun {
    length_lcn: u64,
    offset_lcn: i64,
}

const DATARUN_END: [u8; 1] = [0x00];
fn datarun(input: &[u8]) -> IResult<&[u8], Datarun> {
    do_parse!(input,
        header: split_datarun_header >>
        length_lcn: map!(take!(header.1), length_in_lcn) >>
        offset_lcn: map!(take!(header.0), offset_in_lcn) >>
        (Datarun{length_lcn: length_lcn, offset_lcn: offset_lcn})
    )
}

fn dataruns(input: &[u8]) -> IResult<&[u8], Vec<Datarun>> {
    do_parse!(input,
        data: many_till!(call!(datarun), tag!(DATARUN_END)) >>
            (data.0)
    )
}

fn curr_length(input: &[u8]) -> IResult<&[u8], u16> {
    IResult::Done(input, input.len() as u16)
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
