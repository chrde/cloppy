use std::ffi::OsString;
use gui::Wnd;
use gui::WM_GUI_ACTION;
use winapi::shared::minwindef::WPARAM;
use Message;
use std::ops::Range;
use StateChange;
use std::sync::Arc;
use sql::Arena;

const STEP: usize = 50000;

pub trait Operation {
    fn handle(&mut self, msg: Message);
}

#[derive(Default)]
pub struct FileListing {
    pub wnd: Option<Wnd>,
    files_loaded: usize,
    last_file_loaded: usize,
    cache_count_ahead: usize,
    available_files: usize,
    query: String,
    arena: Arc<Arena>,
}

impl FileListing {
    pub fn new(arena: Arc<Arena>) -> Self {
        FileListing {
            arena,
            cache_count_ahead: STEP / 2,
            ..Default::default()
        }
    }

    fn load_more_data(&mut self, _r: Range<u32>) {
//        let now = Instant::now();
//        let needs_more_data = self.files_loaded < r.start as usize + self.cache_count_ahead;
//        println!("{} {}", r.start, r.end);
//        if self.available_files > self.files_loaded && needs_more_data {
//            println!("load more -before- - items loaded {}, available {}, request {}", self.files_loaded, self.available_files, r.start);
//            println!("load more in {:?} ms", Instant::now().duration_since(now).subsec_nanos() / 1000000);
//            let state = Box::new(State { items, count: 0, status: StateChange::UPDATE });
//            let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
//            wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
//        }
    }

    fn reset(&mut self, req: OsString) {
        self.query = req.to_string_lossy().into_owned();
        self.files_loaded = 0;
        self.last_file_loaded = 0;
        self.available_files = 0;
    }

    fn find_in_tree(&mut self) -> Range<usize> {
        let query = &self.query;
        let from = self.arena.find_by_name(query);
//        let to = self.arena.find_by_name(query);
        let to = self.arena.files.len();
        self.available_files = to - from;
        (from..to)
    }

    fn handle_msg(&mut self, req: OsString) {
        self.reset(req);
//        let now = Instant::now();
        let items = self.find_in_tree();
//        println!("1-found {}, total loaded {} files in {:?} ms", self.available_files, self.files_loaded, Instant::now().duration_since(now).subsec_nanos() / 1000000);
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

pub struct State {
    items: Range<usize>,
    count: u32,
    status: StateChange,
}

impl State {

    pub fn new() -> Self {
        State {
            items: 0..0,
            count: 0,
            status: StateChange::default(),
        }
    }

    pub fn items_start(&self) -> usize {
        self.items.start
    }

    pub fn items_end(&self) -> usize {
        self.items.end
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn status(&self) -> &StateChange {
        &self.status
    }
}
