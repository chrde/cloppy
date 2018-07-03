use crossbeam_channel::internal::channel;
use file_listing::FilesMsg;
use gui;
use gui::Wnd;
use plugin::Plugin;
use plugin::PluginState;
use std::ffi::OsString;
use std::sync::Arc;
use winapi::um::winnt::LPWSTR;

pub struct Dispatcher {
    plugin: Arc<Plugin>,
    state: Arc<PluginState>,
    wnd: Option<Wnd>,
    sender: channel::Sender<UiAsyncMessage>,
}

unsafe impl Send for Dispatcher {}

unsafe impl Sync for Dispatcher {}

impl Dispatcher {
    pub fn new(wnd: Option<Wnd>, plugin: Arc<Plugin>, state: Arc<PluginState>, sender: channel::Sender<UiAsyncMessage>) -> Dispatcher {
        Dispatcher {
            plugin,
            state,
            wnd,
            sender,
        }
    }
}

impl GuiDispatcher for Dispatcher {
    fn set_wnd(&self, wnd: Wnd) {
        unimplemented!()
    }

    fn plugin(&self) -> &Plugin {
        unimplemented!()
    }

    fn state(&self) -> &PluginState {
        unimplemented!()
    }

    fn send_msg(&self, msg: UiSyncMessage) -> UiResult {
        unimplemented!()
    }

    fn send_async_msg(&self, msg: UiAsyncMessage) {
        unimplemented!()
    }
}

pub trait PluginDispatcher {
    fn update_state(&mut self, state: Arc<PluginState>);
}

pub enum UiSyncMessage {
    DrawItem,
    CustomDrawItem,
    PrepareItem,
}

pub enum UiAsyncMessage {
    Ui(OsString),
    Files(FilesMsg),
}

pub enum UiResult {
    Ignore,
    Simple(LPWSTR),
    CustomHandled,
}

pub trait GuiDispatcher {
    fn set_wnd(&self, wnd: Wnd);
    //this should be called by the 'gui' right after creating the wnd, instead of passing it through the channel
    fn plugin(&self) -> &Plugin;
    fn state(&self) -> &PluginState;
    fn send_msg(&self, msg: UiSyncMessage) -> UiResult;
    fn send_async_msg(&self, msg: UiAsyncMessage);
}