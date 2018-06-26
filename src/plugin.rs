use file_listing::file_entity::FileId;
use gui::event::Event;
use winapi::shared::ntdef::LPWSTR;

pub trait Plugin {
    fn draw_item(&self, event: Event) -> DrawResult;
    fn custom_draw_item(&self, event: Event) -> CustomDrawResult;
    fn prepare_item(&self, item_id: usize, state: &State);
    fn handle_message(&self, msg: String) -> Box<State>;
}

pub enum DrawResult {
    IGNORE,
    SIMPLE(LPWSTR),
}

pub enum CustomDrawResult {
    HANDLED,
    IGNORED,
}

#[derive(Default)]
pub struct State {
    count: usize,
    query: String,
}


impl State {
    pub fn new(query: String, count: usize) -> State {
        State {
            query,
            count,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

