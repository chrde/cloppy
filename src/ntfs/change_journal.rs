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
    WinUsnChanges,
};
use winapi::um::winioctl::NTFS_FILE_RECORD_OUTPUT_BUFFER;
use winapi::shared::minwindef::BYTE;
use ntfs::VolumeData;
use std::fs::File;
use std::mem;
use std::path::Path;
use ntfs::FileEntry;
use windows::utils::windows_string;
use ntfs::file_record::parse_file_record;


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
    mft_id: u16,
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
        let mft_id = fr_number as u16;
        let usn = LittleEndian::read_i64(&input[24..]);
        let reason = LittleEndian::read_u32(&input[40..]);
        let flags = LittleEndian::read_u32(&input[52..]);
        let name_length = LittleEndian::read_u16(&input[56..]) as usize;
        let name_offset = LittleEndian::read_u16(&input[58..]) as usize;
        let name = windows_string(&input[name_offset..name_offset + name_length]);
        Ok(UsnRecord { mft_id, fr_number, seq_number, parent_fr_number, reason, name, flags, length, usn })
    }

    fn is_old(&self, file_entry: &FileEntry) -> bool {
        let change = WinUsnChanges::from_bits_truncate(self.reason);
        if file_entry.fr_number != self.fr_number && !change.contains(WinUsnChanges::FILE_DELETE) {
            return true;
        }
        false
    }

    fn into_change(self, entry: FileEntry) -> UsnChange {
        use self::UsnChange::*;
        let change = WinUsnChanges::from_bits_truncate(self.reason);
        if change == WinUsnChanges::CLOSE {
            return IGNORE;
        }
        if entry.fr_number != self.fr_number && !change.contains(WinUsnChanges::FILE_DELETE) {
            return IGNORE;
        }
        if change.contains(WinUsnChanges::FILE_DELETE | WinUsnChanges::FILE_CREATE) {
            return IGNORE;
        }
        if change.contains(WinUsnChanges::FILE_DELETE) {
            return DELETE(self.mft_id);
        }
        if change.contains(WinUsnChanges::FILE_CREATE) {
            return NEW(entry);
        }
        if change.contains(WinUsnChanges::BASIC_INFO_CHANGE & WinUsnChanges::RENAME_NEW_NAME) {
            return UPDATE(entry);
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::UsnChange::*;

    fn new_record(change_reason: WinUsnChanges) -> UsnRecord {
        UsnRecord {
            fr_number: 0,
            mft_id: 0,
            seq_number: 0,
            parent_fr_number: 0,
            reason: change_reason.bits(),
            flags: 0,
            usn: 0,
            length: 0,
            name: "name".to_owned(),
        }
    }

    #[test]
    fn usn_record_ignore_close_only() {
        let record = new_record(WinUsnChanges::CLOSE);
        assert_eq!(IGNORE, record.into_change(FileEntry::default()));
    }

    #[test]
    fn usn_record_to_update() {
        let mut record = new_record(WinUsnChanges::BASIC_INFO_CHANGE);
        assert_eq!(UPDATE(FileEntry::default()), record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::BASIC_INFO_CHANGE | WinUsnChanges::CLOSE);
        assert_eq!(UPDATE(FileEntry::default()), record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME);
        assert_eq!(UPDATE(FileEntry::default()), record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME | WinUsnChanges::CLOSE);
        assert_eq!(UPDATE(FileEntry::default()), record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME | WinUsnChanges::BASIC_INFO_CHANGE | WinUsnChanges::CLOSE);
        assert_eq!(UPDATE(FileEntry::default()), record.into_change(FileEntry::default()));
    }

    #[test]
    fn usn_record_to_delete() {
        let mut record = new_record(WinUsnChanges::FILE_DELETE);
        record.mft_id = 99;
        assert_eq!(DELETE(99), record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::FILE_DELETE | WinUsnChanges::CLOSE);
        record.mft_id = 99;
        assert_eq!(DELETE(99), record.into_change(FileEntry::default()));
    }

    #[test]
    fn usn_record_ignores_create_and_delete_at_once() {
        let mut record = new_record(WinUsnChanges::FILE_DELETE | WinUsnChanges::FILE_CREATE);
        assert_eq!(IGNORE, record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::all());
        assert_eq!(IGNORE, record.into_change(FileEntry::default()));
    }


    #[test]
    fn usn_record_to_create() {
        let mut record = new_record(WinUsnChanges::FILE_DELETE);
        record.mft_id = 99;
        assert_eq!(DELETE(99), record.into_change(FileEntry::default()));
    }

    #[test]
    fn usn_record_ignore_record_with_old_sqn_number() {
        let record = new_record(!WinUsnChanges::FILE_DELETE);
        let mut entry = FileEntry::default();
        entry.fr_number = 1;
        assert_eq!(IGNORE, record.into_change(entry));
    }
}


#[derive(Debug, PartialEq)]
pub enum UsnChange {
    NEW(FileEntry),
    UPDATE(FileEntry),
    DELETE(u16),
    IGNORE,
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

    pub fn get_new_changes(&mut self) -> Result<Vec<UsnChange>, Error> {
        let mut buffer = vec![0u8; self.volume_data.bytes_per_cluster as usize];
        let mut output_buffer = [0u8; mem::size_of::<NTFS_FILE_RECORD_OUTPUT_BUFFER>() + mem::size_of::<BYTE>() * 4096];
        let buffer = read_usn_journal(&self.volume, self.next_usn, self.usn_journal_id, &mut buffer).context(UsnJournalError)?;
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
            let entry = parse_file_record(fr_buffer, self.volume_data);
            usn_records.push(record.into_change(entry));
        }
        self.next_usn = next_usn;
        Ok(usn_records)
    }
}
