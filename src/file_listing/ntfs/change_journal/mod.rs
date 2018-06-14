use crossbeam_channel as channel;
use failure::Error;
use file_listing::FilesMsg;
use file_listing::ntfs::change_journal::usn_journal::UsnJournal;
use Message;
use std::thread;

mod usn_journal;
pub mod usn_record;

pub fn run(sender: channel::Sender<Message>) -> Result<(), Error> {
    thread::Builder::new().name("read journal".to_string()).spawn(move || {
        let volume_path = "\\\\.\\C:";
        let mut journal = UsnJournal::new(volume_path).unwrap();
        loop {
            let changes = journal.get_new_changes().unwrap();
            sender.send(Message::Files(FilesMsg::ChangeJournal(changes)));
        }
    })?;
    Ok(())
}

