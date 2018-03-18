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
    get_usn_journal,
    read_usn_journal,
    UsnJournal as WinJournal,
};
use ntfs::VolumeData;
use std::fs::File;
use std::path::Path;
use windows::get_volume_data;
use ntfs::FileEntry;
use windows::utils::windows_string;

pub struct UsnJournal {
    volume: File,
    volume_data: VolumeData,
    usn_journal_id: u64,
    next_usn: i64,
    buffer: Vec<u8>,
}


#[derive(Getters, Debug)]
pub struct UsnRecord {
    fr_number: u64,
    parent_fr_number: u64,
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
        let (fr_number, parent_fr_number) = match version {
            2 => {
                let fr = LittleEndian::read_u64(&input[8..]);
                let parent_fr = LittleEndian::read_u64(&input[16..]);
                (fr, parent_fr)
            }
            _ => Err(UsnRecordVersionUnsupported(version))?
        };
        let usn = LittleEndian::read_i64(&input[24..]);
        let reason = LittleEndian::read_u32(&input[40..]);
        let flags = LittleEndian::read_u32(&input[52..]);
        let name_length = LittleEndian::read_u16(&input[56..]) as usize;
        let name_offset = LittleEndian::read_u16(&input[58..]) as usize;
        let name = windows_string(&input[name_offset..name_offset + name_length]);
        Ok(UsnRecord { fr_number, parent_fr_number, reason, name, flags, length, usn })
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
        let buffer = read_usn_journal(&self.volume, self.next_usn, self.usn_journal_id, &mut self.buffer).context(UsnJournalError)?;
        let mut usn_records = vec![];
        let next_usn = LittleEndian::read_i64(buffer);
        let mut offset = 8;
        loop {
            if offset == buffer.len() {
                break;
            }
            let record = UsnRecord::new(&buffer[offset..]).context(UsnJournalError)?;
            println!("{:?}", record);
            offset += record.length;
            usn_records.push(record);
        }
        self.next_usn = next_usn;
        Ok(usn_records)
    }
}


