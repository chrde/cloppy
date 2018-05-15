use gui::Wnd;
use gui::WM_GUI_ACTION;
use winapi::shared::minwindef::WPARAM;
use Message;
use StateChange;
use std::sync::Arc;
use sql::Arena;

#[derive(Default)]
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
            ..Default::default()
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
        let state = Box::new(State { items, status: StateChange::NEW });
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
    items: Vec<usize>,
    status: StateChange,
}

impl State {
    pub fn new() -> Self {
        State {
            items: Vec::new(),
            status: StateChange::default(),
        }
    }

    pub fn items(&self) -> &[usize] {
        &self.items
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn status(&self) -> &StateChange {
        &self.status
    }
}
