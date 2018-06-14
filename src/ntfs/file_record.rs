use byteorder::{
    ByteOrder,
    LittleEndian,
};
use ntfs::attributes::DATA;
use ntfs::attributes::parse_attributes;
use ntfs::file_entry::FileEntry;
use ntfs::volume_data::VolumeData;

#[derive(Debug)]
pub struct FileRecordHeader {
    pub fr_number: u32,
    pub seq_number: u16,
    pub flags: u16,
    pub base_record: u64,
    fixup_seq: Vec<u8>,
    attr_offset: usize,
}

fn file_record_header(input: &[u8]) -> Option<FileRecordHeader> {
    if input[..4] == b"FILE"[..] {
        let fixup_offset = LittleEndian::read_u16(&input[0x4..]) as usize;
        let fixup_size = LittleEndian::read_u16(&input[0x06..]) as usize;
        let seq_number = LittleEndian::read_u16(&input[0x10..]) as u16;
        let attr_offset = LittleEndian::read_u16(&input[0x14..]) as usize;
        let flags = LittleEndian::read_u16(&input[0x16..]);
        let base_record = LittleEndian::read_u64(&input[0x20..]);
        let fr_number = LittleEndian::read_u32(&input[0x2C..]);
        let fixup_seq = input[fixup_offset..fixup_offset + 2 * fixup_size].to_vec();
        Some(FileRecordHeader {
            flags,
            fr_number,
            attr_offset,
            seq_number,
            fixup_seq,
            base_record,
        })
    } else {
        None
    }
}

pub fn file_record(buffer: &mut [u8], volume_data: VolumeData) -> Option<FileEntry> {
    match file_record_header(buffer) {
        Some(header) => {
            for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
                buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
            }
            let attributes = parse_attributes(&buffer[header.attr_offset as usize..], DATA);
            Some(FileEntry::new(attributes, header))
        }
        _ => None
    }
}
