use gui::event::Event;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::ntdef::LPWSTR;

pub trait Plugin {
    fn draw_item(&self, file: usize, column: i32) -> ItemDraw;
    fn custom_draw_item(&self, event: Event) -> LRESULT;
    fn prepare_item(&self, item_id: usize, state: &State);
    fn handle_message(&self, msg: String) -> Box<State>;
}

pub enum ItemDraw {
    IGNORE,
    SIMPLE(LPWSTR),
}

#[derive(Default)]
pub struct State {
    items: Vec<ItemIdx>,
    query: String,
}


impl State {
    pub fn new(query: String, items: Vec<ItemIdx>) -> State {
        State {
            query,
            items,
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
