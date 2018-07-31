use actions::exit_app::exit_app;
use actions::focus_on_input_field::focus_on_input_field;
use actions::minimize_to_tray::minimize_to_tray;
use actions::new_input_query::new_input_query;
use actions::restore_windows_position::restore_windows_position;
use actions::save_columns_position::save_columns_position;
use actions::save_windows_position::save_windows_position;
use actions::shortcuts::Shortcut;
use actions::show_files_window::show_files_window;
use errors::failure_to_string;
use failure::Error;
use gui::event::Event;
use gui::Gui;
use actions::restore_columns_position::restore_columns_position;

pub mod shortcuts;
mod new_input_query;
mod save_windows_position;
mod show_files_window;
mod restore_windows_position;
mod minimize_to_tray;
mod focus_on_input_field;
mod exit_app;
mod save_columns_position;
mod restore_columns_position;

#[derive(Debug)]
pub enum Action {
    Simple(SimpleAction),
    Composed(ComposedAction),
}

impl Action {
    pub fn execute(&self, event: Event, gui: &Gui) {
        debug!(&gui.logger(), "ui action" ; "action" => ?self);
        match self {
            Action::Simple(action) => {
                if let Err(e) = action.handler()(event, gui) {
                    error!(&gui.logger(), "ui action failed"; "action" => ?action, "error" => failure_to_string(e));
                }
            }
            Action::Composed(action) => {
                for simple_action in action.simple_actions() {
                    if let Err(e) = simple_action.handler()(event, gui) {
                        error!(&gui.logger(), "ui composed action failed"; "composed action" => ?action, "action" => ?simple_action, "error" => failure_to_string(e));
                        break;
                    }
                }
            }
        }
    }
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
    SaveColumnsPosition,
    RestoreColumnsPosition,
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
            SimpleAction::SaveColumnsPosition => save_columns_position,
            SimpleAction::RestoreColumnsPosition => restore_columns_position,
            SimpleAction::DoNothing => do_nothing,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ComposedAction {
    RestoreWindow,
    ResizeWindowFromSettings,
}

impl ComposedAction {
    pub fn simple_actions(self) -> &'static [SimpleAction] {
        static RESTORE_WINDOW: [SimpleAction; 2] = [SimpleAction::ShowFilesWindow, SimpleAction::FocusOnInputField];
        static RESIZE_WINDOW_FROM_SETTINGS: [SimpleAction; 2] = [SimpleAction::RestoreWindowPosition, SimpleAction::RestoreColumnsPosition];
        match self {
            ComposedAction::RestoreWindow => &RESTORE_WINDOW,
            ComposedAction::ResizeWindowFromSettings => &RESIZE_WINDOW_FROM_SETTINGS,
        }
    }
}

impl From<Shortcut> for Action {
    fn from(shortcut: Shortcut) -> Self {
        match shortcut {
            Shortcut::RestoreWindow => Action::Composed(ComposedAction::RestoreWindow),
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

fn do_nothing(_event: Event, _gui: &Gui) -> Result<(), Error> {
    Ok(())
}
