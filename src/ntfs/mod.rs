use nom::{IResult, le_u16, le_u32};
use self::attributes::*;
use self::attributes::data_attr;
use self::volume_data::VolumeData;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use windows;

mod volume_data;
mod file_entry;
mod attributes;

const END: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

pub struct MftParser {
    file: File,
    volume_data: VolumeData,
    buffer: Vec<u8>,
}

const SPEED_FACTOR: u64 = 4;

impl MftParser {
    pub fn new<P: AsRef<Path>>(volume_path: P) -> Self {
        let file = File::open(volume_path).expect("Failed to open volume handle");
        let volume_data = VolumeData::new(windows::open_volume(&file));
        let buffer = vec![0; volume_data.bytes_per_cluster as usize];
        MftParser { file, volume_data, buffer }
    }

    pub fn parse(&mut self) {
        let fr0 = self.read_mft0();
        println!("{:#?}", fr0);
        use std::time::Instant;
        let mut absolute_lcn_offset = 0i64;
        let now = Instant::now();
        for (i, run) in fr0.dataruns.iter().enumerate() {
            absolute_lcn_offset += run.offset_lcn;
            let absolute_offset = absolute_lcn_offset as u64 * self.volume_data.bytes_per_cluster as u64;
            let file_record_count = run.length_lcn * self.volume_data.clusters_per_fr() as u64;
            println!("datarun {} started", file_record_count);
            for fr in 0..(file_record_count / SPEED_FACTOR) {
                let from = SeekFrom::Start(absolute_offset + SPEED_FACTOR * fr * self.volume_data.bytes_per_file_record as u64);
                self.fill_buffer(from);
                for buff in self.buffer.chunks_mut(self.volume_data.bytes_per_file_record as usize) {
                    MftParser::read_file_record(buff, &self.volume_data, without_dataruns);
                }
            }
            println!("datarun {} finished", i);
            println!("total time {:?}", Instant::now().duration_since(now));
        }
    }

    pub fn read_mft0(&mut self) -> file_entry::FileEntry {
        let from = SeekFrom::Start(self.volume_data.initial_offset());
        self.fill_buffer(from);
        MftParser::read_file_record(&mut self.buffer[0..self.volume_data.bytes_per_file_record as usize], &self.volume_data, with_dataruns)
    }

    fn fill_buffer(&mut self, offset: SeekFrom) {
        self.file.seek(offset).unwrap();
        let fr_size = self.volume_data.bytes_per_file_record as usize;
        let x = &mut self.buffer[..fr_size];
        let file = &mut self.file;
        file.read_exact(x).unwrap();
    }

    fn read_file_record<T>(buffer: &mut [u8], volume_data: &VolumeData, attr_parser: T) -> file_entry::FileEntry
        where T: Fn(&[u8], u32) -> IResult<&[u8], attributes::AttributeType> {
        let res = file_record_header(buffer).to_result().ok();
        match res {
            Some((frn, header)) => {
                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
                }
                match parse_attributes(attr_parser, &buffer[header.attr_offset as usize..]) {
                    IResult::Done(_, r) => {
                        let entry = file_entry::FileEntry::new(r.0, frn);
                        return entry;
                    }
                    _ => {
                        println!("error or incomplete");
                        panic!("cannot parse attributes");
                    }
                }
            }
            _ => return file_entry::FileEntry::default()
        }
    }
}

struct FileRecordHeader {
    fixup_seq: Vec<u8>,
    attr_offset: u16,
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
