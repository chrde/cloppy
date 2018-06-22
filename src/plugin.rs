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
    items: Vec<ItemId>,
    query: String,
}


impl State {
    pub fn new(query: String, items: Vec<ItemId>) -> State {
        State {
            query,
            items,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn items(&self) -> &[ItemId] {
        &self.items
    }

    pub fn count(&self) -> usize {
        self.items().len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ItemId {
    idx: usize,
}

impl ItemId {
    pub fn new(idx: usize) -> ItemId {
        ItemId { idx }
    }

    pub fn id(&self) -> usize {
        self.idx
    }
}
