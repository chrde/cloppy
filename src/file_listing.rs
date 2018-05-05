use gui::Wnd;
use gui::WM_GUI_ACTION;
use winapi::shared::minwindef::WPARAM;
use Message;
use std::ops::Range;
use StateChange;
use std::sync::Arc;
use sql::Arena;

#[derive(Default)]
pub struct FileListing {
    pub wnd: Option<Wnd>,
    available_files: usize,
    query: String,
    arena: Arc<Arena>,
}

impl FileListing {
    pub fn new(arena: Arc<Arena>) -> Self {
        FileListing {
            arena,
            ..Default::default()
        }
    }

    fn find_in_tree(&mut self) -> Range<usize> {
        let query = &self.query;
        let from = self.arena.find_by_name(query);
//        let to = self.arena.find_by_name(query);
        let to = self.arena.files.len();
        self.available_files = to - from;
        (from..to)
    }

    fn handle_msg(&mut self, query: String) {
        self.query = query;
        let items = self.find_in_tree();
        let state = Box::new(State { items, count: self.available_files as u32, status: StateChange::NEW });
        let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
        wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
    }

    pub fn handle(&mut self, msg: Message) {
        match msg {
            Message::START(main_wnd) => self.wnd = Some(main_wnd),
            Message::MSG(v) => self.handle_msg(v.to_string_lossy().into_owned()),
            Message::LOAD(_r) => {},
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
