use byteorder::{ByteOrder, LittleEndian};
use errors::MyErrorKind::UsnRecordVersionUnsupported;
use failure::Error;
use file_listing::file_entity::FileEntity;
use ntfs::file_entry::FileEntry;
use windows::utils::windows_string;

#[derive(Debug, PartialEq)]
pub enum UsnChange {
    NEW(FileEntity),
    UPDATE(FileEntity),
    DELETE(u32),
    IGNORE,
}

#[derive(Debug)]
pub struct UsnRecord {
    pub fr_number: i64,
    mft_id: u32,
    seq_number: u16,
    parent_fr_number: i64,
    reason: u32,
    flags: u32,
    usn: i64,
    pub length: usize,
    name: String,
}

bitflags! {
    struct WinUsnChanges: u32 {
        const FILE_CREATE= 0x00000100;
        const FILE_DELETE= 0x00000200;
        const RENAME_NEW_NAME= 0x00002000;
        const BASIC_INFO_CHANGE= 0x00008000;
        const CLOSE= 0x80000000;
    }
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
        let mft_id = fr_number as u32;
        let usn = LittleEndian::read_i64(&input[24..]);
        let reason = LittleEndian::read_u32(&input[40..]);
        let flags = LittleEndian::read_u32(&input[52..]);
        let name_length = LittleEndian::read_u16(&input[56..]) as usize;
        let name_offset = LittleEndian::read_u16(&input[58..]) as usize;
        let name = windows_string(&input[name_offset..name_offset + name_length]);
        Ok(UsnRecord { mft_id, fr_number, seq_number, parent_fr_number, reason, name, flags, length, usn })
    }

    pub fn into_change(self, entry: FileEntry) -> UsnChange {
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
            return NEW(FileEntity::from_file_entry(entry));
        }
        if change.contains(WinUsnChanges::BASIC_INFO_CHANGE & WinUsnChanges::RENAME_NEW_NAME) {
            return UPDATE(FileEntity::from_file_entry(entry));
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
        let change = UPDATE(FileEntity::from_file_entry(FileEntry::default()));
        assert_eq!(change, record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::BASIC_INFO_CHANGE | WinUsnChanges::CLOSE);
        assert_eq!(change, record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME);
        assert_eq!(change, record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME | WinUsnChanges::CLOSE);
        assert_eq!(change, record.into_change(FileEntry::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME | WinUsnChanges::BASIC_INFO_CHANGE | WinUsnChanges::CLOSE);
        assert_eq!(change, record.into_change(FileEntry::default()));
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