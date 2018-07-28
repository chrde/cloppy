use actions::exit_app::exit_app;
use actions::focus_on_input_field::focus_on_input_field;
use actions::minimize_to_tray::minimize_to_tray;
use actions::new_input_query::new_input_query;
use actions::restore_windows_position::restore_windows_position;
use actions::save_windows_position::save_windows_position;
use actions::shortcuts::Shortcut;
use actions::show_files_window::show_files_window;
use failure::Error;
use gui::event::Event;
use gui::Gui;
use std::iter;
use std::iter::Once;

pub mod shortcuts;
mod new_input_query;
mod save_windows_position;
mod show_files_window;
mod restore_windows_position;
mod minimize_to_tray;
mod focus_on_input_field;
mod exit_app;

#[derive(Debug)]
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
    SaveWindowPosition,
    RestoreWindowPosition,
//    FocusOnItemList,
}

impl SimpleAction {
    pub fn handler(&self) -> impl Fn(Event, &Gui) -> Result<(), Error> {
        match self {
            SimpleAction::ShowFilesWindow => show_files_window,
            SimpleAction::MinimizeToTray => minimize_to_tray,
            SimpleAction::ExitApp => exit_app,
            SimpleAction::NewInputQuery => new_input_query,
            SimpleAction::FocusOnInputField => focus_on_input_field,
            SimpleAction::SaveWindowPosition => save_windows_position,
            SimpleAction::RestoreWindowPosition => restore_windows_position,
            SimpleAction::DoNothing => do_nothing,
        }
    }
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
            ComposedAction::RestoreWindow => vec!(SimpleAction::ShowFilesWindow, SimpleAction::RestoreWindowPosition, SimpleAction::FocusOnInputField).into_iter(),
        }
    }
}

fn do_nothing(_event: Event, _gui: &Gui) -> Result<(), Error> {
    Ok(())
}
