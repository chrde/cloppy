pub use self::file_entry::FileEntry;

pub mod volume_data;
mod mft_parser;
pub mod file_record;
mod file_entry;
mod mft_reader;
mod parse_operation;
mod attributes;


//TODO make this value 'smart' depending on the HD
const FR_AT_ONCE: u64 = 4 * 16;
