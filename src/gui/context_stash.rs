use std::sync::mpsc;
use std::cell::RefCell;
use Message;
use file_listing::State;
use std::sync::Arc;
use sql::Arena;

thread_local!(pub static CONTEXT_STASH: RefCell<Option<ThreadLocalData>> = RefCell::new(None));

pub struct ThreadLocalData {
    sender: mpsc::Sender<Message>,
    pub state: Box<State>,
    pub arena: Arc<Arena>
}

impl ThreadLocalData {
    pub fn new(sender: mpsc::Sender<Message>, arena: Arc<Arena>) -> Self {
        ThreadLocalData {
            sender,
            arena,
            state: Box::new(State::new())
        }
    }
}

pub fn set_state(new_state: Box<State>) {
    CONTEXT_STASH.with(|context_stash| {
        let mut context_stash = context_stash.borrow_mut();
        context_stash.as_mut().unwrap().state = new_state;
    });
}

pub fn send_message(msg: Message) {
    CONTEXT_STASH.with(|context_stash| {
        let context_stash = context_stash.borrow();

        let _ = context_stash.as_ref().unwrap().sender.send(msg);   // Ignoring if closed
    });
}
