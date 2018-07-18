use failure::Error;
use ntfs::file_record::FileRecord;
use ntfs::mft_parser::MftParser;
use ntfs::volume_data::VolumeData;
use ntfs::windows_api::get_volume_data;
use slog::Logger;
use sql::insert_files;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::thread;


fn parse_volume<P: AsRef<Path>>(logger: Logger, path: P) -> Vec<FileRecord> {
    info!(logger, "parse volume"; "status" => "started");
    let (mft, volume) = read_mft(path.as_ref());

    let mut parser = MftParser::new(logger.clone(), &mft, volume);
    let mut reader = parser.new_reader(path, 42);

    let read_thread = thread::Builder::new().name("producer".to_string()).spawn(move || {
        reader.read_all(&mft, volume);
    }).unwrap();
    parser.parse_iocp_buffer();
    read_thread.join().expect("reader panic");
    info!(logger, "parse volume"; "status" => "finished", "files count"=> parser.files.len());
    parser.files
}

fn read_mft<P: AsRef<Path>>(volume_path: P) -> (FileRecord, VolumeData) {
    let mut file = File::open(volume_path).expect("Failed to open volume handle");
    let volume_data = VolumeData::new(get_volume_data(&file).unwrap());
    let mut buffer = vec![0u8; volume_data.bytes_per_file_record as usize];

    file.seek(SeekFrom::Start(volume_data.initial_offset())).unwrap();
    file.read_exact(&mut buffer).unwrap();
    let mft = FileRecord::parse_mft_entry(&mut buffer, volume_data).unwrap();

    (mft, volume_data)
}

pub fn run(parent_logger: Logger) -> Result<(), Error> {
    let volume_path = "\\\\.\\C:";
    let logger = parent_logger.new(o!("plugin" =>"files", "volume" => volume_path.to_string()));
    if !Path::new("./test.db").exists() {
        let files = parse_volume(logger, volume_path);
        insert_files(&files);
    }
    Ok(())
}
