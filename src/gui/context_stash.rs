use std::sync::mpsc;
use std::cell::RefCell;
use Message;
use std::sync::Arc;
use sql::Arena;

thread_local!(pub static CONTEXT_STASH: RefCell<Option<ThreadLocalData>> = RefCell::new(None));

pub struct ThreadLocalData {
    sender: mpsc::Sender<Message>,
    pub arena: Arc<Arena>
}

impl ThreadLocalData {
    pub fn new(sender: mpsc::Sender<Message>, arena: Arc<Arena>) -> Self {
        ThreadLocalData {
            sender,
            arena,
        }
    }
}

pub fn send_message(msg: Message) {
    CONTEXT_STASH.with(|context_stash| {
        let context_stash = context_stash.borrow();

        let _ = context_stash.as_ref().unwrap().sender.send(msg);   // Ignoring if closed
    });
}
