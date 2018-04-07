use std::sync::mpsc;
use std::cell::RefCell;
use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use gui::wnd;
use WndId;
use std::ffi::OsString;

pub struct ThreadLocalData {
    sender: mpsc::Sender<OsString>,
    windows: HashMap<WndId, Arc<Mutex<wnd::Wnd>>>,
}

impl ThreadLocalData {
    pub fn new(sender: mpsc::Sender<OsString>, wnd_count: Option<usize>) -> Self {
        ThreadLocalData {
            sender,
            windows: HashMap::with_capacity(wnd_count.unwrap_or(5))
        }
    }
}

pub fn send_event(event: OsString) {
    CONTEXT_STASH.with(|context_stash| {
        let context_stash = context_stash.borrow();

        let _ = context_stash.as_ref().unwrap().sender.send(event);   // Ignoring if closed
    });
}

thread_local!(pub static CONTEXT_STASH: RefCell<Option<ThreadLocalData>> = RefCell::new(None));