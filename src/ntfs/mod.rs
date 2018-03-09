use byteorder::{
    ByteOrder,
    LittleEndian,
};
use self::attributes::*;
use self::volume_data::VolumeData;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use windows;

mod volume_data;
mod file_entry;
mod attributes;


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
            MftParser::read_file_record(buff, &self.volume_data, FILENAME);
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
        MftParser::read_file_record(&mut self.buffer[0..self.volume_data.bytes_per_file_record as usize], &self.volume_data, DATA)
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
    }

    fn read_file_record(buffer: &mut [u8], volume_data: &VolumeData, last_attr: u32) -> file_entry::FileEntry {
        match file_record_header(buffer) {
            Some(header) => {
                let frn = header.fr_number;
                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
                }
                let attributes = parse_attributes(&buffer[header.attr_offset as usize..], last_attr);
                file_entry::FileEntry::new(attributes, frn)
            }
            _ => return file_entry::FileEntry::default()
        }
    }
}

#[derive(Debug)]
struct FileRecordHeader {
    fr_number: u32,
    fixup_seq: Vec<u8>,
    attr_offset: usize,
}

fn file_record_header(input: &[u8]) -> Option<FileRecordHeader> {
    if input[..4] == b"FILE"[..] {
        let fixup_offset = LittleEndian::read_u16(&input[0x4..]) as usize;
        let fixup_size = LittleEndian::read_u16(&input[0x06..]) as usize;
        let attr_offset = LittleEndian::read_u16(&input[0x14..]) as usize;
        let fr_number = LittleEndian::read_u32(&input[0x2C..]);
        let fixup_seq = input[fixup_offset..fixup_offset + 2 * fixup_size].to_vec();
        Some(FileRecordHeader {
            fr_number,
            attr_offset,
            fixup_seq,
        })
    } else {
        None
    }
}

