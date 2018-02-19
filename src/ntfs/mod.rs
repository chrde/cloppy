use windows;

use std::fs::File;
use nom::{le_u8, le_u16, le_u32, le_u64, IResult};
use std::io::SeekFrom;
use std::io::prelude::*;
use super::FileEntry;
mod volume_data;
mod filename;
mod data;
mod standard;
pub use self::volume_data::VolumeDataRaw;
pub use self::data::Datarun;
use ntfs::filename::{FilenameAttr, filename_attr};
use ntfs::standard::{StandardAttr, standard_attr};
use ntfs::data::{data_attr};

const ATTR_HEADER_SIZE: u32 = 0x10;
//attr_length and attr_offset
const RESIDENT_HEADER_SIZE: u32 = ATTR_HEADER_SIZE + 4 + 2;
//starting_vcn, last_vcn and datarun_offset
const NONRESIDENT_HEADER_SIZE: u32 = ATTR_HEADER_SIZE + 8 + 8 + 2;
const END: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
//pub struct MftParser {
//    handle: File,
//    volume_data: windows::VolumeData,
//    buffer: Vec<u8>,
//}
//
//impl MftParser {
//    pub fn new(handle: File, volume_data: windows::VolumeData) -> Self {
//        let buffer = vec![0; volume_data.bytes_per_cluster as usize];
//        MftParser { handle, volume_data, buffer }
//    }
//
//    pub fn read_mft0(&mut self) -> FileEntry {
//        let initial_offset = self.volume_data.initial_offset();
//        self.handle.seek(SeekFrom::Start(initial_offset)).unwrap();
//        self.handle.read_exact(&mut self.buffer).unwrap();
//        self.read_file_record(with_dataruns);
//        FileEntry::default()
//    }
//
//    fn read_file_record<T>(&mut self, attr_parser: T)
//        where T: Fn(&[u8], u32) -> IResult<&[u8], AttributeType> {
//        let bytesPerSector = self.volume_data.bytes_per_sector;
//        let res = file_record_header(buffer).to_result().ok();
//        match res {
//            Some((frn, header)) => {
//                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
//                    buffer[bytesPerSector * (i + 1) - 2] = *chunk.first().unwrap();
//                    buffer[bytesPerSector * (i + 1) - 1] = *chunk.last().unwrap();
//                }
//                match parse_attributes(attr_parser, &buffer[header.attr_offset as usize..]) {
//                    IResult::Done(_, r) => {
//                        let entry = FileEntry::new(r.0, frn);
//                        return entry;
//                    }
//                    _ => {
//                        println!("error or incomplete");
//                        panic!("cannot parse attributes");
//                    }
//                }
//            }
//            _ => return FileEntry::default()
//        }
//    }
//}

pub fn fixup_buffer(buffer: &mut [u8]) -> FileEntry {
    let bytesPerSector = 512;
    let res = file_record_header(buffer).to_result().ok();
    if res.is_none() {
        println!("wrong");
    }
    match res {
        Some((frn, header)) => {
            for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                buffer[bytesPerSector * (i + 1) - 2] = *chunk.first().unwrap();
                buffer[bytesPerSector * (i + 1) - 1] = *chunk.last().unwrap();
            }
            let attr_parser = if frn == 0 {
                with_dataruns
            } else {
                without_dataruns
            };
            match parse_attributes(attr_parser, &buffer[header.attr_offset as usize..]) {
                IResult::Done(_, r) => {
                    let entry = FileEntry::new(r.0, frn);
                    return entry
                }
                _ => {
                    println!("error or incomplete");
                    panic!("cannot parse attributes");
                }
            }

        },
        _ => return FileEntry::default()
    }
}


#[derive(Debug)]
pub enum AttributeType {
    Standard(StandardAttr),
    Filename(FilenameAttr),
    Data(Vec<Datarun>),
    Ignored,
}

struct FileRecordHeader {
    fixup_seq: Vec<u8>,
    attr_offset: u16,
}
#[derive(Debug)]
pub struct Attribute {
    attr_flags: u16,
    pub attr_type: AttributeType,
}


fn file_record_header(input: &[u8]) -> IResult<&[u8], (u32, FileRecordHeader)> {
    do_parse!(input,
        tag!(b"FILE") >>
        take!(2) >>
        fixup_size: le_u16 >>
        take!(12) >>
        attr_offset: le_u16 >>
        take!(22) >>
        fr_number: le_u32 >>
        fixup_seq: take!(2 * fixup_size) >>
        (fr_number,
        FileRecordHeader{
            attr_offset: attr_offset,
            fixup_seq: fixup_seq.to_vec(),
        })
    )
}

fn parse_attributes<T>(attributes_parser: T, input: &[u8]) -> IResult<&[u8], (Vec<Attribute>, &[u8])>
    where T: Fn(&[u8], u32) -> IResult<&[u8], AttributeType> {
    many_till!(input, call!(attributes, &attributes_parser), tag!(END))
}

fn without_dataruns(input: &[u8], attr_type: u32) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        attribute_type: switch!(value!(attr_type),
                    0x10 => call!(standard_attr) |
                    0x30 => call!(filename_attr) |
                    _ => value!(AttributeType::Ignored)) >>
        (attribute_type)
    )
}

fn with_dataruns(input: &[u8], attr_type: u32) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        attribute_type: switch!(value!(attr_type),
                    0x10 => call!(standard_attr) |
                    0x30 => call!(filename_attr) |
                    0x80 => call!(data_attr) |
                    _ => value!(AttributeType::Ignored)) >>
        (attribute_type)
    )
}

fn attributes<T>(input: &[u8], attr_parser: T) -> IResult<&[u8], Attribute>
    where T: Fn(&[u8], u32) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        current_pos: curr_position >>
        attr_type: le_u32 >>
        attr_length: le_u32 >>
        take!(4) >>
        flags: le_u16 >>
        take!(2) >>
        attr: call!(attr_parser, attr_type) >>
        new_pos: curr_position >>
        take!(attr_length - (current_pos - new_pos)) >>
        (Attribute{
            attr_flags: flags,
            attr_type: attr,
        })
    )
}

fn curr_position(input: &[u8]) -> IResult<&[u8], u32> {
    IResult::Done(input, input.len() as u32)
}
