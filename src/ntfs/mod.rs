use nom::{IResult, le_u16, le_u32};
use self::attributes::*;
use self::attributes::data_attr;
use self::volume_data::VolumeData;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use windows;
use flame;
use byteorder::{
    ByteOrder,
    LittleEndian,
};

mod volume_data;
mod file_entry;
mod attributes;

const END1 : u32 = 0xFFFFFFFF;
const STANDARD: u32 = 0x10;
const FILENAME: u32 = 0x30;
const END: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

pub struct MftParser {
    file: File,
    volume_data: VolumeData,
    buffer1: Vec<u8>,
    buffer: [u8; SPEED_FACTOR as usize * 1024],
    count: u64,
}

//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: u64 = 4 * 16;

impl MftParser {
    //    pub fn new<P: AsRef<Path>>(volume_path: P) -> Self {
    pub fn new(volume_path: &str) -> Self {
//        let file = File::open(volume_path).expect("Failed to open volume handle");
        let file = windows::open_file(volume_path);
        let volume_data = VolumeData::new(windows::open_volume(&file));
        let buffer1 = Vec::with_capacity(SPEED_FACTOR as usize * volume_data.bytes_per_file_record as usize);
        let buffer = [0; SPEED_FACTOR as usize * 1024];
//        let buffer = vec![0; SPEED_FACTOR as usize * volume_data.bytes_per_cluster as usize];
        MftParser { file, volume_data, buffer1, buffer, count: 0 }
    }

    fn parse_chunk(&mut self, offset: u64, chunk_number: u64, size: usize) {
        let from = SeekFrom::Start(offset + SPEED_FACTOR * chunk_number * self.volume_data.bytes_per_file_record as u64);
        self.fill_buffer(from);
        for buff in self.buffer.chunks_mut(self.volume_data.bytes_per_file_record as usize).take(size) {
            MftParser::read_file_record(buff, &self.volume_data, without_dataruns);
            self.count += 1;
        }
    }

    pub fn parse(&mut self, fr0: file_entry::FileEntry) {
        //        let fr0 = self.read_mft0();
//        println!("{:#?}", fr0);
        use std::time::Instant;
        let mut absolute_lcn_offset = 0i64;
        let now = Instant::now();
        for (i, run) in fr0.dataruns.iter().enumerate() {
            absolute_lcn_offset += run.offset_lcn;
            let absolute_offset = absolute_lcn_offset as u64 * self.volume_data.bytes_per_cluster as u64;
            let mut file_record_count = run.length_lcn * self.volume_data.clusters_per_fr() as u64;
//            let mut file_record_count = 2048;
            println!("datarun {} started", file_record_count);

            let full_runs = file_record_count / SPEED_FACTOR;
            let partial_run_size = file_record_count % SPEED_FACTOR;
            for run in 0..full_runs {
                self.parse_chunk(absolute_offset, run, SPEED_FACTOR as usize);
            }
            self.parse_chunk(absolute_offset, full_runs - 1, partial_run_size as usize);
            println!("datarun {} finished", i);
            println!("total time {:?}", Instant::now().duration_since(now));
            println!("total files {:?}", self.count);
        }
    }

    pub fn read_mft0(&mut self) -> file_entry::FileEntry {
        let from = SeekFrom::Start(self.volume_data.initial_offset());
        self.fill_buffer(from);
        MftParser::read_file_record0(&mut self.buffer[0..self.volume_data.bytes_per_file_record as usize], &self.volume_data, with_dataruns)
    }

    fn fill_buffer(&mut self, offset: SeekFrom) {
        self.file.seek(offset).unwrap();
        let buffer = &mut self.buffer;
        let file = &mut self.file;
        let x = Vec::<u32>::with_capacity(buffer.len());
        if x.capacity() == 0 {
            panic!();
        }
        windows::read_file(file, buffer).unwrap();
//        file.read_exact(buffer).unwrap();
    }
    fn read_file_record0<T>(buffer: &mut [u8], volume_data: &VolumeData, attr_parser: T) -> file_entry::FileEntry
        where T: Fn(&[u8], u32) -> IResult<&[u8], attributes::AttributeType> {
        let res = file_record_header(buffer).to_result().ok();
        match res {
            Some(header) => {
                let frn = header.fr_number;
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
    fn read_file_record<T>(buffer: &mut [u8], volume_data: &VolumeData, attr_parser: T) -> file_entry::FileEntry
        where T: Fn(&[u8], u32) -> IResult<&[u8], attributes::AttributeType> {
        let res = file_record_header(buffer).to_result().ok();
//        match res {
//            Some((frn, header)) => {
//                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
//                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
//                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
//                }
//                match parse_attributes(attr_parser, &buffer[header.attr_offset as usize..]) {
//                    IResult::Done(_, r) => {
//                        let entry = file_entry::FileEntry::new(r.0, frn);
//                        return entry;
//                    }
//                    _ => {
//                        println!("error or incomplete");
//                        panic!("cannot parse attributes");
//                    }
//                }
//            }
//            _ => return file_entry::FileEntry::default()
//        }
        file_entry::FileEntry::default()
    }
}

#[derive(Debug)]
struct FileRecordHeader {
    fr_number: u32,
    fixup_seq: Vec<u8>,
    attr_offset: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    const FR0: [u8; 1024] = [70, 73, 76, 69, 48, 0, 3, 0, 80, 245, 122, 254, 24, 0, 0, 0, 1, 0, 1, 0, 56, 0, 1, 0, 184, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 99, 6, 255, 255, 0, 0, 0, 0, 16, 0, 0, 0, 96, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 72, 0, 0, 0, 24, 0, 0, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 0, 0, 0, 104, 0, 0, 0, 0, 0, 24, 0, 0, 0, 3, 0, 74, 0, 0, 0, 24, 0, 1, 0, 5, 0, 0, 0, 0, 0, 5, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 4, 3, 36, 0, 77, 0, 70, 0, 84, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 104, 0, 0, 0, 1, 0, 64, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 83, 8, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 51, 32, 200, 0, 0, 0, 12, 67, 236, 207, 0, 118, 65, 153, 0, 67, 237, 201, 0, 94, 217, 243, 0, 51, 72, 235, 0, 12, 153, 121, 67, 191, 6, 5, 60, 11, 224, 0, 0, 0, 176, 0, 0, 0, 72, 0, 0, 0, 1, 0, 64, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 66, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 49, 67, 118, 24, 3, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 94, 177, 15, 1, 224, 99, 6, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 99, 6];

    #[test]
    fn test_withoutnom1() {
        let x = file_record_header1(&FR0);
        println!("{:?}", x);
    }

    #[test]
    fn test_withnom1() {
        let x = file_record_header(&FR0);
        println!("{:?}", x);
    }

    #[bench]
    fn bench_withoutnom(b: &mut Bencher) {
        b.iter(|| file_record_header1(&FR0));
    }

    #[bench]
    fn bench_withnom(b: &mut Bencher) {
        b.iter(|| file_record_header(&FR0));
    }
}

fn file_record_header1(input: &[u8]) -> Option<FileRecordHeader> {
    if input[..4] == b"FILE"[..] {
        let fixup_offset = LittleEndian::read_u16(&input[0x4..]);
        let fixup_size = LittleEndian::read_u16(&input[0x06..]);
        let attr_offset = LittleEndian::read_u16(&input[0x14..]);
        let fr_number = LittleEndian::read_u32(&input[0x2C..]);
        let fixup_seq = input[fixup_offset as usize..(fixup_offset + 2 * fixup_size) as usize].to_vec();
        Some(FileRecordHeader {
            fr_number,
            attr_offset,
            fixup_seq,
        })
    } else {
        None
    }
}

fn file_record_header(input: &[u8]) -> IResult<&[u8], FileRecordHeader> {
    do_parse!(input,
        tag!(b"FILE") >>
        take!(2) >>
        fixup_size: le_u16 >>
        take!(12) >>
        attr_offset: le_u16 >>
        take!(22) >>
        fr_number: le_u32 >>
        fixup_seq: take!(2 * fixup_size) >>
        (FileRecordHeader{
            fr_number,
            attr_offset: attr_offset,
            fixup_seq: fixup_seq.to_vec(),
        })
    )
}

fn parse_attributes1<T>(attributes_parser: T, input: &[u8]) -> IResult<&[u8], (Vec<Attribute>, &[u8])>
    where T: Fn(&[u8], u32) -> IResult<&[u8], AttributeType> {
    let parsed_attributes: Vec<Attribute> = Vec::with_capacity(2);
    let offset = 0;
    loop {
        let attr_type = LittleEndian::read_u32(input[offset..]);
        if attr_type == END1 || attr_type > FILENAME{
            break
        }
        if attr_type == STANDARD {

        }
    }
    parsed_attributes
}

fn standard_attr_header(input: &[u8]) {
    let length = LittleEndian::read_u32(input[0x04..]);
    let non_resident = LittleEndian::read_u8(input[0x08..]);
    TODO
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

fn attributes1<T>(input: &[u8], attr_parser: T) -> IResult<&[u8], Attribute>
    where T: Fn(&[u8], u32) -> IResult<&[u8], AttributeType> {}

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
