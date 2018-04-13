use byteorder::{LittleEndian, ByteOrder};
use failure::{
    Error,
    ResultExt,
};
use errors::MyErrorKind::*;
use user_settings::{
    Settings,
    UserSettings,
};
use std::fs::OpenOptions;
use windows::{
    get_file_record,
    get_volume_data,
    get_usn_journal,
    read_usn_journal,
    UsnJournal as WinJournal,
    UsnChanges,
};
use winapi::um::winioctl::NTFS_FILE_RECORD_OUTPUT_BUFFER;
use winapi::shared::minwindef::BYTE;
use ntfs::VolumeData;
use std::fs::File;
use std::mem;
use std::path::Path;
use ntfs::FileEntry;
use windows::utils::windows_string;
use ntfs::parse_file_record_basic;


pub struct UsnJournal {
    volume: File,
    volume_data: VolumeData,
    usn_journal_id: u64,
    next_usn: i64,
    buffer: Vec<u8>,
}


#[derive(Debug)]
pub struct UsnRecord {
    fr_number: i64,
    seq_number: u16,
    parent_fr_number: i64,
    reason: u32,
    flags: u32,
    usn: i64,
    length: usize,
    name: String,
}

impl UsnRecord {
    pub fn new(input: &[u8]) -> Result<Self, Error> {
        let length = LittleEndian::read_u32(input) as usize;
        let version = LittleEndian::read_u16(&input[4..]);
        let (seq_number, fr_number, parent_fr_number) = match version {
            2 => {
                let seq_number = LittleEndian::read_u16(&input[14..]);
                let fr = LittleEndian::read_i64(&input[8..]);
                let parent_fr = LittleEndian::read_i64(&input[16..]);
                (seq_number, fr, parent_fr)
            }
            _ => Err(UsnRecordVersionUnsupported(version))?
        };
        let usn = LittleEndian::read_i64(&input[24..]);
        let reason = LittleEndian::read_u32(&input[40..]);
        let flags = LittleEndian::read_u32(&input[52..]);
        let name_length = LittleEndian::read_u16(&input[56..]) as usize;
        let name_offset = LittleEndian::read_u16(&input[58..]) as usize;
        let name = windows_string(&input[name_offset..name_offset + name_length]);
        Ok(UsnRecord { fr_number, seq_number, parent_fr_number, reason, name, flags, length, usn })
    }
}

impl UsnJournal {
    pub fn new<P: AsRef<Path>>(volume_path: P) -> Result<Self, Error> {
        let volume = File::open(volume_path).context(UsnJournalError)?;
        let volume_data = get_volume_data(&volume).map(VolumeData::new).context(UsnJournalError)?;
        let buffer = vec![0u8; volume_data.bytes_per_cluster as usize];
        let WinJournal { usn_journal_id, next_usn } = get_usn_journal(&volume).context(UsnJournalError)?;
        Ok(UsnJournal {
            volume,
            volume_data,
            usn_journal_id,
            next_usn,
            buffer,
        })
    }

    pub fn get_new_changes(&mut self) -> Result<Vec<UsnRecord>, Error> {
        let mut output_buffer = vec![0u8; mem::size_of::<NTFS_FILE_RECORD_OUTPUT_BUFFER>() + mem::size_of::<BYTE>() * 4096];
        let buffer = read_usn_journal(&self.volume, self.next_usn, self.usn_journal_id, &mut self.buffer).context(UsnJournalError)?;
        let mut usn_records = vec![];
        let next_usn = LittleEndian::read_i64(buffer);
        let mut offset = 8;
        loop {
            if offset == buffer.len() {
                break;
            }
            let record = UsnRecord::new(&buffer[offset..]).context(UsnJournalError)?;
            offset += record.length;
            let (fr_buffer, fr_number) = get_file_record(&self.volume, record.fr_number, &mut output_buffer).unwrap();
            let entry = parse_file_record_basic(fr_buffer, self.volume_data);

            let change = UsnChanges::from_bits_truncate(record.reason);
            if change == UsnChanges::CLOSE {
                continue;
            }
            if entry.fr_number != record.fr_number && !change.contains(UsnChanges::FILE_DELETE) {
                continue;
            }
            if record.flags & 0x10 != 0 {
                println!("directory");
            }
            println!("{:?}", change);
            println!("{:?}", record);
            println!("{:?}\n", entry);

            usn_records.push(record);
        }
        self.next_usn = next_usn;
        Ok(usn_records)
    }
}


