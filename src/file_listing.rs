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
use Message;
use rusqlite;
use sql::FileEntity;

pub trait Operation {
    //    fn new(req_rcv: Receiver<OsString>, resp_snd: Sender<OsString>);
    fn handle(&mut self, msg: Message, con: &Connection);
}

pub struct FileListing {
    pub wnd: Option<Wnd>,
    //    resp_send: Sender<OsString>,
//    req_rcv: Receiver<OsString>,
}

impl FileListing {
    fn handle_msg(&mut self, req: OsString, con: &Connection) {
        let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
        let x = req.to_string_lossy().to_string() + "%";
        let (items, next_page) = select_files(con, &x, None).unwrap();
//        let count = count_files(con, &x);
        let count = items.len() as u32;
        let status_bar_message = count.to_string() + " objects found";
        let state = Box::new(State { items, count });
        set_string(STATUS_BAR_CONTENT, status_bar_message);
        wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
    }
}

impl Operation for FileListing {
    fn handle(&mut self, msg: Message, con: &Connection) {
        match msg {
            Message::START(main_wnd) => self.wnd = Some(main_wnd),
            Message::MSG(v) => self.handle_msg(v, con),
            Message::LOAD(r) => {
//                println!("load {} {}", r.start, r.end);
            }
        }
    }
}

#[derive(Default)]
pub struct State {
    items: Vec<FileEntity>,
    count: u32,
}

impl State {
    pub fn get_item(&self, nth: i32) -> &FileEntity {
        &self.items[nth as usize]
    }

    pub fn count(&self) -> u32 {
        self.count
    }
}

