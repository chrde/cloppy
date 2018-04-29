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
use sql::FileNextPage;
use std::sync::mpsc::Sender;

pub trait Operation {
    //    fn new(req_rcv: Receiver<OsString>, resp_snd: Sender<OsString>);
    fn handle(&mut self, msg: Message, con: &Connection);
}

pub struct FileListing {
    pub wnd: Option<Wnd>,
    //    next_page: Option<FileNextPage>,
    files_loaded: u32,
    cache_count_ahead: u32,
    self_snd: Sender<Message>,
    //    resp_send: Sender<OsString>,
//    req_rcv: Receiver<OsString>,
}

impl FileListing {
    pub fn new(self_snd: Sender<Message>, cache_count_ahead: u32) -> Self {
        FileListing {
            wnd: None,
            self_snd,
            files_loaded: 0,
            cache_count_ahead,
        }
    }

    fn update_position(&mut self, next_page: Option<FileNextPage>) {
        if let Some(page) = next_page {
            self.files_loaded += page.page_size;
        } else {
            println!("FIX ME");
//            unreachable!("UI should not ask for more data at the end");
        }
    }

    fn reset_position(&mut self) {
        self.files_loaded = 0;
    }

    fn handle_msg(&mut self, req: OsString, con: &Connection) {
        self.reset_position();
        let x = req.to_string_lossy().to_string() + "%";
        let (items, next_page) = select_files(con, &x, None).unwrap();
        println!("has more? {}", next_page.is_some());
        self.update_position(next_page);
        let count = count_files(con, &x);
        let status_bar_message = count.to_string() + " objects found";
        let state = Box::new(State { items, count });
        set_string(STATUS_BAR_CONTENT, status_bar_message);
        let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
        wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
    }
}

impl Operation for FileListing {
    fn handle(&mut self, msg: Message, con: &Connection) {
        match msg {
            Message::START(main_wnd) => self.wnd = Some(main_wnd),
            Message::MSG(v) => self.handle_msg(v, con),
            Message::LOAD(r) => {
                println!("load {} {}", r.start, r.end);
                if self.files_loaded <= r.start + self.cache_count_ahead {
                    println!("load more - current items {}, request {}", self.files_loaded, r.start);
                }
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

