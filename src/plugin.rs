use gui::event::Event;
use winapi::shared::ntdef::LPWSTR;

pub trait Plugin: Sync + Send {
    fn draw_item(&self, event: Event) -> DrawResult;
    fn custom_draw_item(&self, event: Event) -> CustomDrawResult;
    fn prepare_item(&self, item_id: usize, state: &PluginState);
    fn handle_message(&self, msg: &str) -> usize;
}

pub trait PluginState: Sync + Send {
    fn count(&self) -> usize;
    fn query(&self) -> &str;
}

pub enum DrawResult {
    IGNORE,
    SIMPLE(LPWSTR),
}

pub enum CustomDrawResult {
    HANDLED,
    IGNORED,
}

pub struct Dummy;

impl PluginState for Dummy {
    fn count(&self) -> usize {
        unimplemented!()
    }

    fn query(&self) -> &str {
        unimplemented!()
    }
}

impl PluginState for State {
    fn count(&self) -> usize {
        self.count
    }

    fn query(&self) -> &str {
        &self.query
    }
}

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

}

