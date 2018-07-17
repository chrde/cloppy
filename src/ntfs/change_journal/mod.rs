use crossbeam_channel as channel;
use dispatcher::UiAsyncMessage;
use failure::Error;
use file_listing::FilesMsg;
use ntfs::change_journal::usn_journal::UsnJournal;
pub use self::usn_record::UsnChange;
use std::thread;

mod usn_journal;
mod usn_record;

pub fn run(sender: channel::Sender<UiAsyncMessage>) -> Result<(), Error> {
    thread::Builder::new().name("read journal".to_string()).spawn(move || {
        let volume_path = "\\\\.\\C:";
        let mut journal = UsnJournal::new(volume_path).unwrap();
        loop {
            let changes = journal.get_new_changes().unwrap();
            sender.send(UiAsyncMessage::Files(FilesMsg::ChangeJournal(changes)));
        }
    })?;
    Ok(())
}

