use byteorder::{
    ByteOrder,
    LittleEndian,
};
use ntfs::attributes::*;
use ntfs::volume_data::VolumeData;

const DOS_NAMESPACE: u8 = 2;

#[derive(Debug, Default)]
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

#[derive(Default, Debug)]
pub struct FileRecord {
    pub data_attr: DataAttr,
    pub name_attrs: Vec<FilenameAttr>,
    pub standard_attr: StandardAttr,
    pub header: FileRecordHeader,
}

impl FileRecord {
    pub fn parse_mft_entry(buffer: &mut [u8], volume_data: VolumeData) -> Option<FileRecord> {
        match file_record_header(buffer) {
            Some(header) => {
                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
                }
                let attributes = parse_attributes(&buffer[header.attr_offset as usize..], DATA);
                Some(FileRecord::new(attributes, header))
            }
            _ => None
        }
    }

    pub fn is_unused(&self) -> bool {
        self.header.flags % 2 == 0 || self.name_attrs.is_empty()
    }

    pub fn is_directory(&self) -> bool {
        self.header.flags == 3
    }

    pub fn has_name(&self) -> bool {
        !(self.name_attrs.len() == 1 && self.name_attrs[0].namespace == DOS_NAMESPACE)
    }

    pub fn is_candidate_for_fixes(&self) -> bool {
        self.header.base_record != 0 && self.is_directory()
    }

    pub fn requires_name_fix(&self) -> bool {
        self.is_directory() && !self.has_name() && self.header.base_record == 0
    }

    pub fn fr_number(&self) -> i64 {
        self.header.fr_number as i64 | (self.header.seq_number as i64) << 48
    }

    pub fn new(attrs: Vec<Attribute>, header: FileRecordHeader) -> Self {
        let mut result = FileRecord::default();
        result.header = header;
        let mut standard_count = 0;
        let mut data_count = 0;
        //TODO handle attribute flags (e.g: sparse or compressed)
        let entry = attrs.into_iter().fold(result, |mut acc, attr| {
            match attr.attr_type {
                AttributeType::Standard(val) => {
                    standard_count += 1;
                    acc.standard_attr = val;
                }
                AttributeType::Filename(val) => {
                    acc.name_attrs.push(val);
                }
                AttributeType::Data(val) => {
                    data_count += 1;
                    acc.data_attr = val;
                }
            }
            acc
        });
        assert!(1 >= standard_count, "Record {} got {} standard_attr", entry.fr_number() as u32, standard_count);
        assert!(1 >= data_count, "Record {} got {} data_attr", entry.fr_number() as u32, data_count);
        entry
    }
}