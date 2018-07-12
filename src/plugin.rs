use gui::event::Event;
use winapi::shared::ntdef::LPWSTR;
use std::any::Any;

pub trait Plugin: Sync + Send {
    fn draw_item(&self, event: Event, state: &State) -> DrawResult;
    fn custom_draw_item(&self, event: Event, state: &State) -> CustomDrawResult;
    fn prepare_item(&self, item_id: usize, state: &mut State);
    fn handle_message(&self, msg: &str) -> State;
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

pub trait PluginState: Any + Sync + Send {
    fn any_ref(&self) -> &Any;
    fn any_mut(&mut self) -> &mut Any;
}

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

    pub fn plugin_state<T: 'static>(&self) -> Option<&T> {
        let state = self.plugin_state.any_ref();
        state.downcast_ref::<T>()
    }

    pub fn plugin_state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let state = self.plugin_state.any_mut();
        state.downcast_mut::<T>()
    }
}

