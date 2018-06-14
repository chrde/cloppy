use failure::Error;
use ntfs::file_entry::FileEntry;
use ntfs::file_record::file_record;
use ntfs::mft_parser::MftParser;
use ntfs::volume_data::VolumeData;
use rusqlite::Connection;
use sql::insert_files;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::thread;
use windows;


fn parse_volume<P: AsRef<Path>>(path: P) -> Vec<FileEntry> {
    let (mft, volume) = read_mft(path.as_ref());

    let mut parser = MftParser::new(&mft, volume);
    let mut reader = parser.new_reader(path, 42);

    let read_thread = thread::Builder::new().name("producer".to_string()).spawn(move || {
        reader.read_all(&mft, volume);
    }).unwrap();
    parser.parse_iocp_buffer();
    read_thread.join().expect("reader panic");
    parser.files
}

fn read_mft<P: AsRef<Path>>(volume_path: P) -> (FileEntry, VolumeData) {
    let mut file = File::open(volume_path).expect("Failed to open volume handle");
    let volume_data = VolumeData::new(windows::get_volume_data(&file).unwrap());
    let mut buffer = vec![0u8; volume_data.bytes_per_file_record as usize];

    file.seek(SeekFrom::Start(volume_data.initial_offset())).unwrap();
    file.read_exact(&mut buffer).unwrap();
    let mft = file_record(&mut buffer, volume_data).unwrap();

    (mft, volume_data)
}

pub fn run(con: &mut Connection) -> Result<(), Error> {
    let volume_path = "\\\\.\\C:";
    let parse = false;
    if parse {
        let files = parse_volume(volume_path);
        insert_files(con, &files);
    }
    println!("usn journal  listening...");
    Ok(())
}
