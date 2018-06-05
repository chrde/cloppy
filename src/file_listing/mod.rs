use gui::Wnd;
use gui::WM_GUI_ACTION;
use winapi::shared::minwindef::WPARAM;
use Message;
use StateChange;
use std::sync::Arc;
use sql::Arena;
use twoway;

pub mod file_type_icon;

pub struct FileListing {
    pub wnd: Option<Wnd>,
    last_query: String,
    arena: Arc<Arena>,
    items_current_search: Vec<usize>,
}

impl FileListing {
    pub fn new(arena: Arc<Arena>) -> Self {
        FileListing {
            arena,
            last_query: String::new(),
            wnd: None,
            items_current_search: Vec::new(),
        }
    }

    fn handle_msg(&mut self, query: String) {
        let items = if query.starts_with(&self.last_query) {
            self.arena.search_by_name(&query, self.items_current_search.iter().cloned())
        } else {
            self.arena.search_by_name(&query, 0..self.arena.file_count())
        };
        self.last_query = query;
        self.items_current_search = items.clone();
        let state = Box::new(State { query: self.last_query.clone(), search_length: self.last_query.len(), items, status: StateChange::NEW });
        let wnd = self.wnd.as_mut().expect("Didnt receive START msg with main_wnd");
        wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
    }

    pub fn handle(&mut self, msg: Message) {
        match msg {
            Message::START(main_wnd) => self.wnd = Some(main_wnd),
            Message::MSG(v) => self.handle_msg(v.to_string_lossy().into_owned()),
            Message::LOAD(_r) => {}
        }
    }
}

#[derive(Default)]
pub struct State {
    status: StateChange,
    items: Vec<usize>,
    query: String,
    search_length: usize,
}

impl State {
    pub fn new() -> Self {
        State::default()
    }


    pub fn matches(&self, name: &str) -> Vec<Match> {
        let mut result = Vec::new();

        let mut curr_pos = 0;
        if self.query.len() > 0 {
            while let Some(rel_pos) = twoway::find_str(&name[curr_pos..], &self.query) {
                if rel_pos > 0 {
                    result.push(Match { matched: false, init: curr_pos, end: curr_pos + rel_pos });
                }
                curr_pos += rel_pos + self.query.len();
                result.push(Match { matched: true, init: rel_pos, end: curr_pos });
            }
        }
        if curr_pos != name.len() {
            result.push(Match { matched: false, init: curr_pos, end: name.len() });
        }
        result
    }

    pub fn items(&self) -> &[usize] {
        &self.items
    }

    pub fn count(&self) -> usize {
        self.items().len()
    }

    pub fn status(&self) -> &StateChange {
        &self.status
    }
}

#[derive(Debug)]
pub struct Match {
    pub matched: bool,
    pub init: usize,
    pub end: usize,
}
