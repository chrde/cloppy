pub use self::volume_data::VolumeData;
pub use self::file_entry::FileEntry;
pub use self::attributes::FILENAME;

mod volume_data;
pub mod mft_parser;
pub mod change_journal;
mod file_record;
mod file_entry;
mod mft_reader;
pub mod parse_operation;
mod attributes;


//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: u64 = 4 * 16;

