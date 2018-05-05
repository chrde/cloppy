use std::sync::mpsc;
use std::cell::RefCell;
use Message;

thread_local!(pub static CONTEXT_STASH: RefCell<Option<ThreadLocalData>> = RefCell::new(None));

pub struct ThreadLocalData {
    sender: mpsc::Sender<Message>,
}

impl ThreadLocalData {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
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
