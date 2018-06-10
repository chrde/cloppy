use file_listing::list::item::Match;
use gui::event::Event;
use winapi::shared::ntdef::LPWSTR;
use winapi::shared::windef::RECT;

pub trait Plugin {
    fn get_draw_info(&self, event: Event, file: usize, column: i32) -> ItemDraw;
    fn prepare_item(&self, item_id: usize, state: &State);
    fn handle_message(&self, msg: String) -> Box<State>;
}

pub enum ItemDraw {
    IGNORE,
    SIMPLE(LPWSTR),
    DETAILED(Vec<SuperMatch>),
}

#[derive(Default)]
pub struct State {
    status: StateChange,
    items: Vec<ItemIdx>,
    query: String,
}


#[derive(Debug)]
pub struct SuperMatch {
    pub matched: bool,
    pub init: usize,
    pub end: usize,
    pub text: Vec<u16>,
}

impl State {
    pub fn new(query: String, items: Vec<ItemIdx>) -> State {
        State {
            query,
            items,
            status: StateChange::NEW,
        }
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

pub enum StateChange {
    NEW,
    UPDATE,
}

impl Default for StateChange {
    fn default() -> Self {
        StateChange::NEW
    }
}

#[derive(Clone, Debug)]
pub struct ItemIdx(usize);

impl ItemIdx {
    pub fn new(idx: usize) -> ItemIdx {
        ItemIdx(idx)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}
