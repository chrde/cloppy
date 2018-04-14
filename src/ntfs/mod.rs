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


//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: u64 = 4 * 16;

pub fn start() -> Result<(), Error> {
    parse_operation::run()
}
