pub use ntfs::change_journal::usn_journal::UsnJournal;
pub use self::usn_record::UsnChange;
pub use self::usn_record::UsnRecord;

mod usn_journal;
mod usn_record;
