use file_listing::file_entity::FileId;
use gui::event::Event;
use std::sync::Arc;
use winapi::shared::ntdef::LPWSTR;

pub trait Plugin {
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

pub struct State {
    count: usize,
    query: String,
    plugin: Arc<Plugin>,
}

impl State {
    pub fn new<T: Into<String>>(query: T, count: usize, plugin: Arc<Plugin>) -> State {
        State {
            query: query.into(),
            count,
            plugin,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn plugin(&self) -> &Arc<Plugin> {
        &self.plugin
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

