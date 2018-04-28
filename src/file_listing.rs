use std::ffi::OsString;
use rusqlite::Connection;
use sql::select_files;
use gui::Wnd;
use gui::WM_GUI_ACTION;
use gui::set_string;
use gui::STATUS_BAR_CONTENT;
use ntfs::FileEntry;
use winapi::shared::minwindef::WPARAM;
use winapi::um::winnt::LPWSTR;
use sql::count_files;

pub trait Operation {
    //    fn new(req_rcv: Receiver<OsString>, resp_snd: Sender<OsString>);
    fn handle(&mut self, req: OsString, con: &Connection, wnd: &Wnd);
}

pub struct FileListing {
    //    resp_send: Sender<OsString>,
//    req_rcv: Receiver<OsString>,
}

impl Operation for FileListing {
    fn handle(&mut self, req: OsString, con: &Connection, wnd: &Wnd) {
        let x = req.to_string_lossy().to_string() + "%";
        let items = select_files(con, &x).unwrap();
        let count = count_files(con, &x);
        let status_bar_message = count.to_string() + " objects found";
        let state = Box::new(State {items, count});
        set_string(STATUS_BAR_CONTENT, status_bar_message);
        wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
    }
}

#[derive(Default)]
pub struct State {
    items: Vec<Entry>,
    count: u32,
}

pub struct Entry {
    values: Vec<Vec<u16>>
}

impl Entry {
    pub fn new(values: Vec<Vec<u16>>) -> Self {
        Entry {values}
    }
    pub fn get_value(&self, nth: i32) -> LPWSTR {
        self.values[nth as usize].as_ptr() as LPWSTR
    }
}

impl State {
    pub fn get_item(&self, nth: i32) -> &Entry {
        &self.items[nth as usize]
    }

    pub fn count(&self) -> u32 {
        self.count
    }
}

