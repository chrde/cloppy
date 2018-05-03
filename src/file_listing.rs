use std::ffi::OsString;
use rusqlite::Connection;
use sql::select_files;
use gui::Wnd;
use gui::WM_GUI_ACTION;
use winapi::shared::minwindef::WPARAM;
use sql::count_files;
use Message;
use sql::FileKey;
use sql::Query;
use std::ops::Range;
use std::collections::btree_set::Range as RangeT;
use StateChange;
use std::time::Instant;
use std::time::Duration;
use std::fmt;
use std::collections::BTreeSet;
use std::collections::Bound::Included;
use std::sync::Arc;
use sql::Arena;

const STEP: usize = 6000;

pub trait Operation {
    //    fn new(req_rcv: Receiver<OsString>, resp_snd: Sender<OsString>);
    fn handle(&mut self, msg: Message);
}

#[derive(Default)]
pub struct FileListing {
    pub wnd: Option<Wnd>,
    files_loaded: usize,
    last_file_loaded: FileKey,
    cache_count_ahead: usize,
    available_files: usize,
    files: BTreeSet<FileKey>,
    arena: Arc<Arena>,
    //    resp_send: Sender<OsString>,
//    req_rcv: Receiver<OsString>,
}

impl FileListing {
    pub fn new(cache_count_ahead: usize, files: BTreeSet<FileKey>, arena: Arc<Arena>) -> Self {
        FileListing {
            files,
            arena,
            cache_count_ahead,
            ..Default::default()
        }
    }

    /*fn update(&mut self, next_page: Query) {
        self.query = next_page;
        if let Some(page) = self.query.next() {
            self.files_loaded += page.page_size;
        } else {
            println!("FIX ME");
//            unreachable!("UI should not ask for more data at the end");
        }
    }*/

    fn load_more_data(&mut self, r: Range<u32>) {
        let now = Instant::now();
        let needs_more_data = self.files_loaded < r.start as usize + self.cache_count_ahead;
//        println!("{} {}", r.start, r.end);
        if self.available_files > self.files_loaded && needs_more_data {
//            println!("load more - items loaded {}, available {}, request {}", self.files_loaded, self.available_files, r.start);
//            let (items, next_query) = select_files(con, &self.query).unwrap();
            let loaded = self.files_loaded;
            let (last, count, items) = {
                let items = self.find_in_tree().take(STEP).map(|f| f.clone()).collect::<Vec<FileKey>>();
                let last = (*items.last().unwrap()).clone();
                let count = items.len();
                (last, count, items)
            };
            self.last_file_loaded = last;
//            self.last_file_loaded = items.last().unwrap().clone();
            self.files_loaded += count;
//            self.update(next_query);
            println!("load more in {:?} ms", Instant::now().duration_since(now).subsec_nanos() / 1000000);
            let items = items.iter().map(|f| f.position()).collect::<Vec<usize>>();
            let state = Box::new(State { items, count: 0, status: StateChange::UPDATE });
            let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
            wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
        }
    }

    fn reset(&mut self, req: OsString) {
        self.files_loaded = 0;
        self.last_file_loaded = FileKey::new(req.to_string_lossy().to_string(), 0);
        self.available_files = 0;
    }

    fn find_in_tree(&mut self) -> RangeT<FileKey> {
//        let target_to = FileKey1 { name: self.query.query().to_owned(), id: <u32>::max_value() };
        let mut range = self.files.range(self.last_file_loaded.clone()..);
        range
//        self.files.range((Included(&target_from), Included(&target_to)))
    }

    fn handle_msg(&mut self, req: OsString) {
        self.reset(req);
        let now = Instant::now();
        let loaded = self.files_loaded;
        let (items, count) = {
            let mut range = self.find_in_tree();
            let count = range.clone().count();
            let items = range.take(STEP).map(|f| f.clone()).collect::<Vec<FileKey>>();
            (items, count)
        };
        self.available_files = count;
        self.files_loaded += items.len();
        self.last_file_loaded = items.last().unwrap().clone();
//        let count = items.len() as u32;
        println!("1-found {}, total loaded {} files in {:?} ms", count, self.files_loaded, Instant::now().duration_since(now).subsec_nanos() / 1000000);
//        let now = Instant::now();
//        let count = count_files(con, &self.query.query());
//        let (items, next_page) = select_files(con, &self.query).unwrap();
//        println!("{} {} has more? {}", 0, self.files.len(), next_page.has_more());
//        println!("2-found {} files in {:?} ms", count, Instant::now().duration_since(now).subsec_nanos() / 1000000);
//        self.update(next_page);
        let items = items.iter().map(|f| f.position()).collect::<Vec<usize>>();
        let state = Box::new(State { items, count: self.available_files as u32, status: StateChange::NEW });
        let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
        wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
    }
}

impl Operation for FileListing {
    fn handle(&mut self, msg: Message) {
        match msg {
            Message::START(main_wnd) => self.wnd = Some(main_wnd),
            Message::MSG(v) => self.handle_msg(v),
            Message::LOAD(r) => self.load_more_data(r),
        }
    }
}

#[derive(Default)]
pub struct State {
    items: Vec<usize>,
    count: u32,
    status: StateChange,
}

impl State {
    pub fn get_item(&self, nth: i32) -> Option<usize> {
//        self.items[nth as usize]
        self.items.get(nth as usize).map(|i| *i)
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
