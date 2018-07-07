use gui::event::Event;
use winapi::shared::ntdef::LPWSTR;

pub trait Plugin: Sync + Send {
    fn draw_item(&self, event: Event) -> DrawResult;
    fn custom_draw_item(&self, event: Event) -> CustomDrawResult;
    fn prepare_item(&self, item_id: usize, state: &State);
    fn handle_message(&self, msg: &str) -> usize;
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
    pub fn new<T: Into<String>>(query: T, count: usize) -> State {
        State {
            query: query.into(),
            count,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}

