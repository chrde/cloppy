use failure::Error;
use gui::event::Event;
use gui::Gui;
use winapi::um::winuser::SW_HIDE;

pub fn minimize_to_tray(_event: Event, gui: &Gui) -> Result<(), Error> {
    gui.wnd().show(SW_HIDE)
}