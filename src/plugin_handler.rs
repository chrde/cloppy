use crossbeam_channel::internal::channel;
use dispatcher::UiAsyncMessage;
use file_listing::FileListing;
use gui::WM_GUI_ACTION;
use gui::Wnd;
use plugin::Plugin;
use plugin::State;
use settings::UserSettings;
use std::sync::Arc;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::WPARAM;
use actions::SimpleAction;
use actions::Action;

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

    pub fn run_forever(&mut self, receiver: channel::Receiver<UiAsyncMessage>, mut settings: UserSettings) {
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
                    let action_ptr = Box::into_raw(Box::new(Action::from(SimpleAction::NewPluginState)));
                    let state_ptr = Box::into_raw(Box::new(state));
                    self.wnd.post_message(WM_GUI_ACTION, state_ptr as WPARAM, action_ptr as LPARAM);
                }
                UiAsyncMessage::UpdateSettings(update) => {
                    let new_settings = settings.update_settings(update).unwrap();
                    let new_settings_ptr = Box::into_raw(Box::new(new_settings));
                    let action_ptr = Box::into_raw(Box::new(Action::from(SimpleAction::NewSettings)));
                    self.wnd.post_message(WM_GUI_ACTION, new_settings_ptr as WPARAM, action_ptr as LPARAM);
                },
                UiAsyncMessage::Start(_) => unreachable!(),
            }
        }
    }
}

