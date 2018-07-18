use byteorder::{ByteOrder, LittleEndian};
use errors::MyErrorKind::UsnRecordVersionUnsupported;
use failure::Error;
use ntfs::file_record::FileRecord;
use ntfs::windows_api::windows_string;

#[derive(Debug, PartialEq)]
pub enum UsnChange {
    NEW(FileRecord),
    UPDATE(FileRecord),
    DELETE(UsnRecord),
    IGNORE,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UsnRecord {
    pub fr_number: i64,
    pub mft_id: u32,
    pub seq_number: u16,
    pub parent_fr_number: i64,
    pub reason: u32,
    pub flags: u32,
    pub usn: i64,
    pub length: usize,
    pub name: String,
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

    pub fn into_change(self, entry: FileRecord) -> UsnChange {
        use self::UsnChange::*;
        let change = WinUsnChanges::from_bits_truncate(self.reason);
        if change == WinUsnChanges::CLOSE {
            return IGNORE;
        }
        if entry.fr_number() != self.fr_number && !change.contains(WinUsnChanges::FILE_DELETE) {
            return IGNORE;
        }
        if change.contains(WinUsnChanges::FILE_DELETE | WinUsnChanges::FILE_CREATE) {
            return IGNORE;
        }
        if change.contains(WinUsnChanges::FILE_DELETE) {
            return match self.flags {
                0x16 => DELETE(self),
                0x30 => DELETE(self),
                _ => IGNORE,
            };
        }
        if change.contains(WinUsnChanges::FILE_CREATE) {
            return NEW(entry);
        }
        if change.contains(WinUsnChanges::BASIC_INFO_CHANGE & WinUsnChanges::RENAME_NEW_NAME) {
            return UPDATE(entry);
        }
        unreachable!()
    }

    pub fn is_dir(&self) -> bool {
        self.flags == 0x16
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
            flags: 0x30,
            usn: 0,
            length: 0,
            name: "name".to_owned(),
        }
    }

    #[test]
    fn usn_record_ignore_close_only() {
        let record = new_record(WinUsnChanges::CLOSE);
        assert_eq!(IGNORE, record.into_change(FileRecord::default()));
    }

    #[test]
    #[ignore]
    fn usn_record_to_update() {
        let mut record = new_record(WinUsnChanges::BASIC_INFO_CHANGE);
        let change = UPDATE(FileRecord::default());
        assert_eq!(change, record.into_change(FileRecord::default()));

        record = new_record(WinUsnChanges::BASIC_INFO_CHANGE | WinUsnChanges::CLOSE);
        assert_eq!(change, record.into_change(FileRecord::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME);
        assert_eq!(change, record.into_change(FileRecord::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME | WinUsnChanges::CLOSE);
        assert_eq!(change, record.into_change(FileRecord::default()));

        record = new_record(WinUsnChanges::RENAME_NEW_NAME | WinUsnChanges::BASIC_INFO_CHANGE | WinUsnChanges::CLOSE);
        assert_eq!(change, record.into_change(FileRecord::default()));
    }

    #[test]
    fn usn_record_to_delete_file() {
        let mut record = new_record(WinUsnChanges::FILE_DELETE);
        record.mft_id = 99;
        assert_eq!(DELETE(record.clone()), record.into_change(FileRecord::default()));

        record = new_record(WinUsnChanges::FILE_DELETE | WinUsnChanges::CLOSE);
        record.mft_id = 99;
        assert_eq!(DELETE(record.clone()), record.into_change(FileRecord::default()));
    }

    #[test]
    fn usn_record_to_delete_dir() {
        let mut record = new_record(WinUsnChanges::FILE_DELETE);
        record.mft_id = 99;
        record.flags = 0x16;
        assert_eq!(DELETE(record.clone()), record.into_change(FileRecord::default()));

        record = new_record(WinUsnChanges::FILE_DELETE | WinUsnChanges::CLOSE);
        record.mft_id = 99;
        record.flags = 0x16;
        assert_eq!(DELETE(record.clone()), record.into_change(FileRecord::default()));
    }

    #[test]
    fn usn_record_ignores_deleted_other_than_dir_or_file() {
        let mut record = new_record(WinUsnChanges::FILE_DELETE);
        record.mft_id = 99;
        record.flags = 0x20;
        assert_eq!(IGNORE, record.into_change(FileRecord::default()));
    }

    #[test]
    fn usn_record_ignores_create_and_delete_at_once() {
        let mut record = new_record(WinUsnChanges::FILE_DELETE | WinUsnChanges::FILE_CREATE);
        assert_eq!(IGNORE, record.into_change(FileRecord::default()));

        record = new_record(WinUsnChanges::all());
        assert_eq!(IGNORE, record.into_change(FileRecord::default()));
    }

    #[test]
    fn usn_record_ignore_record_with_old_sqn_number() {
        let record = new_record(!WinUsnChanges::FILE_DELETE);
        let mut entry = FileRecord::default();
        entry.header.fr_number = 1;
        assert_eq!(IGNORE, record.into_change(entry));
    }
}