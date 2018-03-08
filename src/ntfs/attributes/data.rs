use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use nom::{IResult, le_u16};
use std::io::Cursor;
use super::AttributeType;
use super::NONRESIDENT_HEADER_SIZE;

const DATARUN_END: [u8; 1] = [0x00];

#[derive(Debug, PartialEq)]
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

fn split_datarun_header(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
    bits!(input,
        pair!(
            take_bits!(u8, 4),
            take_bits!(u8, 4)))
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_calculates_offset_in_lcn() {
        assert_eq!(0x1280, offset_in_lcn(&[0x80, 0x12]));
        assert_eq!(-0x8012, offset_in_lcn(&[0x12, 0x80]), "greater than 0x7F is negative");
    }

    #[test]
    fn it_calculates_length_in_lcn() {
        assert_eq!(25, length_in_lcn(&[25]));
        assert_eq!(0x1234, length_in_lcn(&[0x34, 0x12]));
        assert_eq!(0x8877665544332211, length_in_lcn(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99]), "skips after the 8th bit");
    }

    #[test]
    fn it_splits_datarun_header() {
        let input = [0x45, 0x99, 0x99];

        let remainder = [0x99, 0x99];
        let result = (4, 5);

        assert_eq!(split_datarun_header(&input[..]), IResult::Done(&remainder[..], result));
    }

    #[test]
    fn it_parses_a_single_datarun() {
        let input = [
            0x21,
            0x18,
            0x34, 0x56,
            0x99, 0x99];

        let remainder = [0x99, 0x99];
        let result = Datarun {
            length_lcn: 0x18,
            offset_lcn: 0x5634,
        };

        assert_eq!(datarun(&input[..]), IResult::Done(&remainder[..], result));
    }

    #[test]
    fn it_parses_fragmented_file() {
        let fragmented_file = [0x31, 0x38, 0x73, 0x25, 0x34, 0x32, 0x14, 0x01, 0xE5, 0x11, 0x02, 0x31, 0x42, 0xAA, 0x00, 0x03, 0x00, 0x99, 0x99];

        let remainder = [0x99, 0x99];
        let result = vec![
            Datarun { length_lcn: 0x38, offset_lcn: 0x342573 },
            Datarun { length_lcn: 0x114, offset_lcn: 0x211E5 },
            Datarun { length_lcn: 0x42, offset_lcn: 0x300AA },
        ];

        assert_eq!(dataruns(&fragmented_file[..]), IResult::Done(&remainder[..], result));
    }

    #[test]
    fn it_parses_scrambled_file() {
        let scrambled_file = [0x11, 0x30, 0x60, 0x21, 0x10, 0x00, 0x01, 0x11, 0x20, 0xE0, 0x00, 0x99, 0x99];

        let remainder = [0x99, 0x99];
        let result = vec![
            Datarun { length_lcn: 0x30, offset_lcn: 0x60 },
            Datarun { length_lcn: 0x10, offset_lcn: 0x100 },
            Datarun { length_lcn: 0x20, offset_lcn: -0xE0 },
        ];

        assert_eq!(dataruns(&scrambled_file[..]), IResult::Done(&remainder[..], result));
    }

    #[test]
    fn it_parses_sparse_file() {
        let sparse_file = [0x11, 0x30, 0x20, 0x01, 0x60, 0x11, 0x10, 0x30, 0x00, 0x99, 0x99];

        let remainder = [0x99, 0x99];
        let result = vec![
            Datarun { length_lcn: 0x30, offset_lcn: 0x20 },
            Datarun { length_lcn: 0x60, offset_lcn: 0x0 },
            Datarun { length_lcn: 0x10, offset_lcn: 0x30 },
        ];

        assert_eq!(dataruns(&sparse_file[..]), IResult::Done(&remainder[..], result));
    }

    #[test]
    #[ignore]
    fn it_parses_compressed_file() {
        unimplemented!();
    }

    #[test]
    fn it_parses_datarun_attr() {
        let b = [
            0, 0, 0, 0, 0, 0, 0, 0, 255, 83, 8, 0, 0, 0, 0, 0,
            64, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0,
            51, 32, 200, 0, 0, 0, 12,
            67, 236, 207, 0, 118, 65, 153, 0, 67, 237, 201, 0, 94, 217, 243, 0, 51, 72, 235, 0, 12, 153, 121, 67, 191, 6, 5, 60, 11, 224, 0, 0,
            99, 99];

        let remainder = [99, 99];
        let result = AttributeType::Data(vec![
            Datarun { length_lcn: 51232, offset_lcn: 786432 },
            Datarun { length_lcn: 53228, offset_lcn: 10043766 },
            Datarun { length_lcn: 51693, offset_lcn: 15980894 },
            Datarun { length_lcn: 60232, offset_lcn: 7969036 },
            Datarun { length_lcn: 329407, offset_lcn: 14682940 }
        ]);

        assert_eq!(data_attr(&b[..]), IResult::Done(&remainder[..], result));
    }
}
