use actions::shortcuts::Shortcut;
use gui::event::Event;
use winapi::um::winuser::SW_SHOW;

pub mod shortcuts;


pub enum Action {
    ShowFilesWindow,
    //    NewInputQuery,
//    ExitApp,
//    MinimizeToTray,
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