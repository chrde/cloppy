use ntfs::VolumeData;
use ntfs::FileEntry;
use byteorder::{
    ByteOrder,
    LittleEndian,
};
use ntfs::attributes::parse_attributes;
use ntfs::FILENAME;
use ntfs::attributes::DATA;

#[derive(Debug)]
pub struct FileRecordHeader {
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

fn file_record(buffer: &mut [u8], volume_data: VolumeData, last_attr: u32) -> FileEntry {
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

pub fn parse_file_record(buffer: &mut [u8], volume_data: VolumeData) -> FileEntry {
    file_record(buffer, volume_data, FILENAME)
}

pub fn parse_fr0(buffer: &mut [u8], volume_data: VolumeData) -> FileEntry {
    file_record(buffer, volume_data, DATA)
}