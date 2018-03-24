use byteorder::{
    ByteOrder,
    LittleEndian,
};
use self::attributes::*;
pub use self::volume_data::VolumeData;
pub use self::file_entry::FileEntry;
pub use self::attributes::FILENAME;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use windows;
use std::path::Path;
use std::thread;
use std::time;
use windows::async_io::AsyncReader;

mod volume_data;
mod file_entry;
mod attributes;


//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: u64 = 4 * 16;

pub fn read_mft<P: AsRef<Path>>(volume_path: P) -> (FileEntry, VolumeData) {
    let mut file = File::open(volume_path).expect("Failed to open volume handle");
    let volume_data = VolumeData::new(windows::get_volume_data(&file).unwrap());
    let mut buffer = vec![0u8; volume_data.bytes_per_file_record as usize];

    file.seek(SeekFrom::Start(volume_data.initial_offset())).unwrap();
    file.read_exact(&mut buffer).unwrap();
    let mft = parse_file_record_0(&mut buffer, volume_data);

    (mft, volume_data)
}

pub fn read_all(mft: &FileEntry, volume_data: VolumeData, async_reader: &mut AsyncReader) {
    use std::time::Instant;
    let now = Instant::now();
    let mut absolute_lcn_offset = 0i64;
    for (i, run) in mft.dataruns.iter().enumerate() {
        absolute_lcn_offset += run.offset_lcn;
        let absolute_offset = absolute_lcn_offset as u64 * volume_data.bytes_per_cluster as u64;
        let mut file_record_count = run.length_lcn * volume_data.clusters_per_fr() as u64;
        println!("datarun {} started", file_record_count);

        let full_runs = file_record_count / SPEED_FACTOR;
        let partial_run_size = file_record_count % SPEED_FACTOR;
        for run in 0..full_runs {
            let offset = absolute_offset + SPEED_FACTOR * run * volume_data.bytes_per_file_record as u64;
            async_reader.read(offset).unwrap();
        }
        let offset = absolute_offset + SPEED_FACTOR * (full_runs - 1) * volume_data.bytes_per_file_record as u64;
        async_reader.read(offset).unwrap();
        println!("datarun {} finished. Partial time {:?}", i, Instant::now().duration_since(now));
    }
    println!("total time {:?}", Instant::now().duration_since(now));
    thread::sleep(time::Duration::from_millis(5000));
    async_reader.finish();
}

pub fn parse_file_record_basic(buffer: &mut [u8], volume_data: VolumeData) -> FileEntry {
    parse_file_record(buffer, volume_data, FILENAME)
}

pub fn parse_file_record_0(buffer: &mut [u8], volume_data: VolumeData) -> FileEntry {
    parse_file_record(buffer, volume_data, DATA)
}

fn parse_file_record(buffer: &mut [u8], volume_data: VolumeData, last_attr: u32) -> FileEntry {
    match file_record_header(buffer) {
        Some(header) => {
            for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
                buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
            }
            let attributes = parse_attributes(&buffer[header.attr_offset as usize..], last_attr);
            FileEntry::new(attributes, header.fr_number, header.seq_number)
        }
        _ => return FileEntry::default()
    }
}

#[derive(Debug)]
struct FileRecordHeader {
    fr_number: u32,
    fixup_seq: Vec<u8>,
    seq_number: u16,
    attr_offset: usize,
}

fn file_record_header(input: &[u8]) -> Option<FileRecordHeader> {
    if input[..4] == b"FILE"[..] {
        let fixup_offset = LittleEndian::read_u16(&input[0x4..]) as usize;
        let fixup_size = LittleEndian::read_u16(&input[0x06..]) as usize;
        let seq_number = LittleEndian::read_u16(&input[0x10..]) as u16;
        let attr_offset = LittleEndian::read_u16(&input[0x14..]) as usize;
        let fr_number = LittleEndian::read_u32(&input[0x2C..]);
        let fixup_seq = input[fixup_offset..fixup_offset + 2 * fixup_size].to_vec();
        Some(FileRecordHeader {
            fr_number,
            attr_offset,
            seq_number,
            fixup_seq,
        })
    } else {
        None
    }
}

