pub use self::file_entry::FileEntry;
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
const FR_AT_ONCE: u64 = 4 * 16;

pub fn start() -> Result<(), Error> {
    parse_operation::run()
}
