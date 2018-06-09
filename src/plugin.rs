use gui::event::Event;
use winapi::shared::windef::RECT;

pub trait Plugin {
    fn draw_item(&self, event: Event, positions: [RECT; 3]);
    fn prepare_item(&self, event: Event, state: &State);
    fn handle_message(&self, msg: String) -> Box<State>;
}

#[derive(Default)]
pub struct State {
    status: StateChange,
    items: Vec<ItemIdx>,
    query: String,
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
