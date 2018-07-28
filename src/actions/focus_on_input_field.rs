use failure::Error;
use gui::event::Event;
use gui::Gui;

pub fn focus_on_input_field(_event: Event, gui: &Gui) -> Result<(), Error> {
    gui.input_search().wnd().set_focus()
}
