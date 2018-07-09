use gui::event::Event;
use winapi::shared::ntdef::LPWSTR;

pub trait Plugin: Sync + Send {
    fn draw_item(&self, event: Event, state: &State) -> DrawResult;
    fn custom_draw_item(&self, event: Event, state: &State) -> CustomDrawResult;
    fn prepare_item(&self, item_id: usize, state: &mut State);
    fn handle_message(&self, msg: &str) -> usize;
    fn default_plugin_state(&self) -> Box<PluginState>;
}

pub enum DrawResult {
    IGNORE,
    SIMPLE(LPWSTR),
}

pub enum CustomDrawResult {
    HANDLED,
    IGNORED,
}

pub trait PluginState: Sync + Send {}

pub struct State {
    count: usize,
    query: String,
    plugin_state: Box<PluginState>,
}

impl State {
    pub fn new<T: Into<String>>(query: T, count: usize, plugin_state: Box<PluginState>) -> State {
        State {
            query: query.into(),
            count,
            plugin_state,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}

