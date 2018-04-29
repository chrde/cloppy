use std::ffi::OsString;
use rusqlite::Connection;
use sql::select_files;
use gui::Wnd;
use gui::WM_GUI_ACTION;
use winapi::shared::minwindef::WPARAM;
use sql::count_files;
use Message;
use sql::FileEntity;
use sql::Query;
use std::ops::Range;
use StateChange;

pub trait Operation {
    //    fn new(req_rcv: Receiver<OsString>, resp_snd: Sender<OsString>);
    fn handle(&mut self, msg: Message, con: &Connection);
}

pub struct FileListing {
    pub wnd: Option<Wnd>,
    query: Query,
    files_loaded: u32,
    cache_count_ahead: u32,
    //    resp_send: Sender<OsString>,
//    req_rcv: Receiver<OsString>,
}

impl FileListing {
    pub fn new(cache_count_ahead: u32) -> Self {
        FileListing {
            query: Query::default(),
            wnd: None,
            files_loaded: 0,
            cache_count_ahead,
        }
    }

    fn update(&mut self, next_page: Query) {
        println!("next page{:?}", next_page);
        self.query = next_page;
        if let Some(page) = self.query.next() {
            self.files_loaded += page.page_size;
        } else {
            println!("FIX ME");
//            unreachable!("UI should not ask for more data at the end");
        }
    }

    fn load_more_data(&mut self, r: Range<u32>, con: &Connection){
        let needs_more_data = self.files_loaded < r.start + self.cache_count_ahead;
        if self.query.has_more() && needs_more_data {
            println!("load more - current items {}, request {}", self.files_loaded, r.start);
            let (items, next_query) = select_files(con, &self.query).unwrap();
            self.update(next_query);
            let state = Box::new(State { items, count: 0, status: StateChange::UPDATE });
            let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
            wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
        }
    }

    fn reset(&mut self, req: OsString) {
        let req = req.to_string_lossy().to_string();
        self.query = Query::new(req);
        self.files_loaded = 0;
    }

    fn handle_msg(&mut self, req: OsString, con: &Connection) {
        self.reset(req);
        let count = count_files(con, &self.query.query());
        let (items, next_page) = select_files(con, &self.query).unwrap();
        println!("has more? {}", next_page.has_more());
        self.update(next_page);
        let state = Box::new(State { items, count, status: StateChange::NEW });
        let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
        wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
    }
}

impl Operation for FileListing {
    fn handle(&mut self, msg: Message, con: &Connection) {
        match msg {
            Message::START(main_wnd) => self.wnd = Some(main_wnd),
            Message::MSG(v) => self.handle_msg(v, con),
            Message::LOAD(r) => self.load_more_data(r, con),
        }
    }
}

#[derive(Default)]
pub struct State {
    items: Vec<FileEntity>,
    count: u32,
    status: StateChange,
}

impl State {
    pub fn get_item(&self, nth: i32) -> Option<&FileEntity> {
        self.items.get(nth as usize)
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn status(&self) -> &StateChange {
        &self.status
    }

    pub fn update_with(&mut self, other: State) {
        self.items.extend(other.items);
    }
}
