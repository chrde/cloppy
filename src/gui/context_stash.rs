use crossbeam_channel as channel;
use Message;
use std::cell::RefCell;

thread_local!(pub static CONTEXT_STASH: RefCell<Option<ThreadLocalData>> = RefCell::new(None));

pub struct ThreadLocalData {
    sender: channel::Sender<Message>,
}

impl ThreadLocalData {
    pub fn new(sender: channel::Sender<Message>) -> Self {
        ThreadLocalData {
            sender,
        }
    }
}

pub fn send_message(msg: Message) {
    CONTEXT_STASH.with(|context_stash| {
        let context_stash = context_stash.borrow();
        let _ = context_stash.as_ref().unwrap().sender.send(msg);   // Ignoring if closed
    });
}
