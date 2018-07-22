use actions::shortcuts::Shortcut;
use gui::event::Event;
use gui::msg::Msg;
use gui::Wnd;
pub use self::new_input_query::new_input_query;
use std::iter;
use std::iter::Once;
use winapi::um::winuser::MSG;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;

pub mod shortcuts;
mod new_input_query;

pub enum Action {
    Simple(SimpleAction),
    Composed(ComposedAction),
}

#[derive(Copy, Clone, Debug)]
pub enum SimpleAction {
    ShowFilesWindow,
    NewInputQuery,
    ExitApp,
    MinimizeToTray,
    DoNothing,
    FocusOnInputField,
//    FocusOnItemList,
}

#[derive(Copy, Clone, Debug)]
pub enum ComposedAction {
    RestoreWindow,
}

impl From<Shortcut> for Action {
    fn from(shortcut: Shortcut) -> Self {
        match shortcut {
            Shortcut::RestoreWindow => ComposedAction::RestoreWindow.into(),
        }
    }
}

impl From<SimpleAction> for Action {
    fn from(action: SimpleAction) -> Self {
        Action::Simple(action)
    }
}

impl From<ComposedAction> for Action {
    fn from(action: ComposedAction) -> Self {
        Action::Composed(action)
    }
}

impl IntoIterator for SimpleAction {
    type Item = SimpleAction;
    type IntoIter = Once<SimpleAction>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        iter::once(self)
    }
}

impl IntoIterator for ComposedAction {
    type Item = SimpleAction;
    type IntoIter = ::std::vec::IntoIter<SimpleAction>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        match self {
            ComposedAction::RestoreWindow => vec!(SimpleAction::ShowFilesWindow, SimpleAction::FocusOnInputField).into_iter(),
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

pub fn focus_on_input_field(wnd: &Wnd) {
    wnd.set_focus();
}