use crossbeam_channel::internal::channel;
use file_listing::FilesMsg;
use gui::Wnd;
use plugin::Plugin;
use plugin::State;
use std::sync::Arc;
use winapi::um::winnt::LPWSTR;

pub struct GuiDispatcher {
    plugin: Arc<Plugin>,
    state: Box<State>,
    sender: channel::Sender<UiAsyncMessage>,
}

impl GuiDispatcher {
    pub fn new(plugin: Arc<Plugin>, state: Box<State>, sender: channel::Sender<UiAsyncMessage>) -> GuiDispatcher {
        GuiDispatcher {
            plugin,
            state,
            sender,
        }
    }
    pub fn plugin(&self) -> &Plugin {
        &*self.plugin
    }

    pub fn state(&self) -> &State {
        &*self.state
    }

    pub fn prepare_item(&mut self, item_id: usize) {
        let state = &mut self.state;
        self.plugin.prepare_item(item_id, state);
    }

    pub fn set_state(&mut self, state: Box<State>) {
        self.state = state;
    }

//    pub fn send_msg(&self, msg: UiSyncMessage) -> UiResult {
//        unimplemented!()
//    }

    pub fn send_async_msg(&self, msg: UiAsyncMessage) {
        self.sender.send(msg);
    }
}

pub enum UiSyncMessage {
    DrawItem,
    CustomDrawItem,
    PrepareItem,
}

pub enum UiAsyncMessage {
    Start(Wnd),
    Ui(String),
    Files(FilesMsg),
}

pub enum UiResult {
    Ignore,
    Simple(LPWSTR),
    CustomHandled,
}
