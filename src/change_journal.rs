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
    UsnJournal as WinJournal,
};
use ntfs::VolumeData;
use std::fs::File;
use std::path::Path;
use windows::get_volume_data;

struct UsnJournal {
    volume: File,
    volume_data: VolumeData,
    usn_journal_id: u64,
    next_usn: i64,
    buffer: Vec<u8>,
}

impl UsnJournal {
    pub fn new<P: AsRef<Path>>(volume_path: P) -> Result<Self, Error> {
        let volume = File::open(volume_path).context(UsnJournalError)?;
        let volume_data = get_volume_data(&volume).map(VolumeData::new).context(UsnJournalError)?;
        let buffer = vec![0u8; volume_data.bytes_per_cluster as usize];
        let WinJournal{usn_journal_id,next_usn}  = get_usn_journal(&volume).context(UsnJournalError)?;
        Ok(UsnJournal {
            volume,
            volume_data,
            usn_journal_id,
            next_usn,
            buffer,
        })
    }

    pub fn get_new_changes(&mut self) {

    }
}

//pub fn last_usn_journal(settings: &UserSettings) -> Result<u64, Error> {
//    Ok(settings.get(Settings::DbFile)
//        .and_then(|s| {
//            let f = OpenOptions::new().read(true).open(s).context(UsnJournalError)?;
//            usn_journal_id(&f)
//        })
//        .context(UsnJournalError)?)
//}
