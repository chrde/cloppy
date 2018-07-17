mod volume_data;
mod mft_parser;
mod file_record;
pub mod file_entry;
mod mft_reader;
pub mod parse_operation;
mod attributes;
pub mod change_journal;


//TODO make this value 'smart' depending on the HD
const FR_AT_ONCE: u64 = 4 * 16;
