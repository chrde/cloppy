use failure::{
    Error,
    ResultExt,
};
use file_listing::ntfs::change_journal::usn_journal::UsnJournal;
use file_listing::ntfs::change_journal::usn_record::UsnChange;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

mod usn_journal;
mod usn_record;

pub fn run(sender: mpsc::Sender<Vec<UsnChange>>) -> Result<(), Error> {
    thread::Builder::new().name("read journal".to_string()).spawn(move || {
        let volume_path = "\\\\.\\C:";
        let mut journal = UsnJournal::new(volume_path).unwrap();
        loop {
            let changes = journal.get_new_changes().unwrap();
            sender.send(changes).unwrap();
        }
    })?;
    Ok(())
}

pub fn debug(receiver: mpsc::Receiver<Vec<UsnChange>>) -> Result<(), Error> {
    thread::Builder::new().name("read journal debug".to_string()).spawn(move || {
        loop {
            let changes = match receiver.recv() {
                Ok(e) => e,
                Err(_) => {
                    println!("Channel closed. Probably UI thread exit.");
                    return;
                }
            };
            for change in changes {
                match change {
                    UsnChange::DELETE(id) => println!("DELETE {}", id),
                    UsnChange::UPDATE(file) => println!("UPDATE {:?}", file),
                    UsnChange::NEW(file) => println!("NEW {:?}", file),
                    UsnChange::IGNORE => {}
                }
            }
        }
    })?;
    Ok(())
}
