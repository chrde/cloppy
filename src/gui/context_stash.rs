use std::sync::mpsc;
use std::cell::RefCell;
use std::collections::HashMap;
use gui::wnd;
use std::ffi::OsString;
use gui::WndId;

thread_local!(pub static CONTEXT_STASH: RefCell<Option<ThreadLocalData>> = RefCell::new(None));

pub struct ThreadLocalData {
    sender: mpsc::Sender<OsString>,
    windows: HashMap<WndId, wnd::Wnd>,
}

impl ThreadLocalData {
    pub fn new(sender: mpsc::Sender<OsString>, wnd_count: Option<usize>) -> Self {
        ThreadLocalData {
            sender,
            windows: HashMap::with_capacity(wnd_count.unwrap_or(5)),
        }
    }
}

pub fn apply_on_window<F, R>(id: WndId, f: F) -> R
    where F: Fn(&wnd::Wnd) -> R {
    CONTEXT_STASH.with(|context_stash| {
        let context_stash = context_stash.borrow();
        let ref thread_local_data = context_stash.as_ref().unwrap();
        let wnd = thread_local_data.windows.get(&id).unwrap();
        f(wnd)
    })
}

pub fn add_window(id: WndId, wnd: wnd::Wnd) {
    CONTEXT_STASH.with(|context_stash| {
        let mut context_stash = context_stash.borrow_mut();

        let old_wnd = context_stash.as_mut().unwrap().windows.insert(id, wnd);
        assert!(old_wnd.is_none());
    });
}

pub fn send_event(event: OsString) {
    CONTEXT_STASH.with(|context_stash| {
        let context_stash = context_stash.borrow();

        let _ = context_stash.as_ref().unwrap().sender.send(event);   // Ignoring if closed
    });
}
