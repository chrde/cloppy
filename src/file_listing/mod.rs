use file_listing::files::Files;
use file_listing::files::ItemIdx;
use gui::WM_GUI_ACTION;
use gui::Wnd;
use Message;
use StateChange;
use std::sync::Arc;
use winapi::shared::minwindef::WPARAM;

mod file_type_icon;
pub mod list;
pub mod file_entity;
pub mod files;

pub struct FileListing {
    pub wnd: Option<Wnd>,
    last_query: String,
    arena: Arc<Files>,
    items_current_search: Vec<ItemIdx>,
}

impl FileListing {
    pub fn new(arena: Arc<Files>) -> Self {
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
            self.arena.new_search_by_name(&query)
        };
        self.last_query = query;
        self.items_current_search = items.clone();
        let state = Box::new(State { query: self.last_query.clone(), items, status: StateChange::NEW });
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
    items: Vec<ItemIdx>,
    query: String,
}

impl State {
    pub fn new() -> Self {
        State::default()
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn items(&self) -> &[ItemIdx] {
        &self.items
    }

    pub fn count(&self) -> usize {
        self.items().len()
    }

    pub fn status(&self) -> &StateChange {
        &self.status
    }
}
