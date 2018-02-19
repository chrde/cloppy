use super::AttributeType;
use byteorder::ReadBytesExt;
use byteorder::LittleEndian;
use std::io::Cursor;
use nom::{le_u8, le_u16, le_u32, le_u64, IResult};
use super::NONRESIDENT_HEADER_SIZE;
const DATARUN_END: [u8; 1] = [0x00];
#[derive(Debug)]
pub struct Datarun {
    pub length_lcn: u64,
    pub offset_lcn: i64,
}


pub fn data_attr(input: &[u8]) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        take!(16) >>
        datarun_offset: le_u16 >>
        skipped_values: value!(datarun_offset as u32 - NONRESIDENT_HEADER_SIZE) >>
        take!(skipped_values) >>
        data: dataruns >>
        (AttributeType::Data(data))
    )
}

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
    } else {
        result
    }
}

