use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct UsnJournal {
    pub usn_journal_id: u64,
    pub next_usn: i64,
}

bitflags! {
    pub struct WinUsnChanges: u32 {
        const FILE_CREATE= 0x00000100;
        const FILE_DELETE= 0x00000200;
        const RENAME_NEW_NAME= 0x00002000;
        const BASIC_INFO_CHANGE= 0x00008000;
        const CLOSE= 0x80000000;
    }
}

impl fmt::Display for WinUsnChanges {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hi!")
    }
}

#[repr(C)]
pub struct ReadUsnJournalDataV0 {
    start: i64,
    reason_mask: u32,
    return_only_on_close: u32,
    timeout: u64,
    bytes_to_wait_for: u64,
    usn_journal_id: u64,
}

impl ReadUsnJournalDataV0 {
    pub fn new(start: i64, usn_journal_id: u64) -> Self {
        ReadUsnJournalDataV0 {
            start,
            reason_mask: WinUsnChanges::all().bits(),
            return_only_on_close: 1,
            timeout: 1,
            bytes_to_wait_for: 1,
            usn_journal_id,
        }
    }
}