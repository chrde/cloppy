use actions::shortcuts::Shortcut;
use gui::event::Event;
use gui::msg::Msg;
pub use self::new_input_query::new_input_query;
use winapi::um::winuser::MSG;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;

pub mod shortcuts;
mod new_input_query;


pub enum Action {
    ShowFilesWindow,
    NewInputQuery,
    ExitApp,
    MinimizeToTray,
    DoNothing,
}

impl From<Shortcut> for Action {
    fn from(shortcut: Shortcut) -> Self {
        match shortcut {
            Shortcut::ShowFilesWindow => Action::ShowFilesWindow,
        }
    }
}

pub fn show_files_window(event: Event) {
    event.wnd().show(SW_SHOW);
    event.wnd().set_as_foreground();
}


pub fn minimize_to_tray(event: Event) {
    event.wnd().show(SW_HIDE);
}

pub fn exit_app() {
    MSG::post_quit(0);
}