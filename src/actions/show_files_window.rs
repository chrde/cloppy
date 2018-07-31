use failure::Error;
use gui::event::Event;
use gui::Gui;
use winapi::um::winuser::SW_RESTORE;

pub fn show_files_window(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    let wnd = gui.wnd();
    wnd.show(SW_RESTORE)
        .and_then(|_| wnd.set_as_foreground())
}