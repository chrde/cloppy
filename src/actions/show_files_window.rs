use failure::Error;
use gui::event::Event;
use gui::Gui;
use winapi::um::winuser::SW_SHOW;

pub fn show_files_window(_event: Event, gui: &Gui) -> Result<(), Error> {
    let wnd = gui.wnd();
    wnd.show(SW_SHOW)
        .and_then(|_| wnd.set_as_foreground())
}