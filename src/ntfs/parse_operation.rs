use ntfs::file_record::parse_fr0;
use ntfs::FileEntry;
use ntfs::mft_parser::MftParser;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::thread;
use windows;
use ntfs::volume_data::VolumeData;
use failure::Error;
use sql::insert_files;
use rusqlite::Connection;


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
    let mft = parse_fr0(&mut buffer, volume_data);

    (mft, volume_data)
}

pub fn run(con: &mut Connection) -> Result<(), Error> {
    let volume_path = "\\\\.\\C:";
//    let mut sql_con = sql::main();
    {
        let files = parse_volume(volume_path);
        insert_files(con, &files);
    }
    println!("usn journal  listening...");
    Ok(())
//    let mut journal = UsnJournal::new(volume_path)?;
//    let read_journal: JoinHandle<Result<(), Error>> = thread::Builder::new().name("read journal".to_string()).spawn(move || {
//        loop {
//            let tx = sql_con.transaction().unwrap();
//            let changes = journal.get_new_changes()?;
//            for change in changes {
//                match change {
//                    UsnChange::DELETE(id) => { delete_file(&tx, id) }
//                    UsnChange::UPDATE(entry) => { update_file(&tx, &entry) }
//                    UsnChange::NEW(entry) => { upsert_file(&tx, &entry) }
//                    UsnChange::IGNORE => {}
//                }
//            }
//            tx.commit().unwrap();
//        }
//    })?;
//    read_journal.join().unwrap().unwrap();
//    Ok(())
}