use crossbeam_channel::internal::channel;
use dispatcher::UiAsyncMessage;
use file_listing::FileListing;
use gui::WM_GUI_ACTION;
use gui::Wnd;
use plugin::Plugin;
use plugin::State;
use std::sync::Arc;
use winapi::shared::minwindef::WPARAM;

pub struct PluginHandler {
    pub files: Arc<FileListing>,
    prev_state: State,
    pub wnd: Wnd,
}

impl PluginHandler {
    pub fn new(wnd: Wnd, files: Arc<FileListing>, initial_state: State) -> PluginHandler {
        PluginHandler {
            files,
            prev_state: initial_state,
            wnd,
        }
    }

    pub fn run_forever(&mut self, receiver: channel::Receiver<UiAsyncMessage>) {
        loop {
            let msg = match receiver.recv() {
                Some(e) => e,
                None => {
                    println!("Channel closed. Probably UI thread exit.");
                    return;
                }
            };
            match msg {
                UiAsyncMessage::Files(msg) => self.files.on_message(msg),
                UiAsyncMessage::Ui(msg) => {
                    let state = self.files.handle_message(&msg, &self.prev_state);
                    self.prev_state = state.clone();
                    println!("{}", state.count());
                    self.wnd.post_message(WM_GUI_ACTION, Box::into_raw(Box::new(state)) as WPARAM);
                }
                UiAsyncMessage::Start(_) => unreachable!(),
            }
        }
    }
}
