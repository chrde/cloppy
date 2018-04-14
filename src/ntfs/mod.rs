//pub use self::volume_data::VolumeData;
pub use self::file_entry::FileEntry;
//pub use self::attributes::FILENAME;
use failure::Error;

mod volume_data;
mod mft_parser;
mod change_journal;
mod file_record;
mod file_entry;
mod mft_reader;
mod parse_operation;
mod attributes;

pub fn start() -> Result<(), Error> {
    parse_operation::run()
}
