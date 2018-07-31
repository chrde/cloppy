use failure::Error;
use gui::event::Event;
use gui::Gui;
use gui::msg::Msg;
use winapi::um::winuser::MSG;

pub fn exit_app(_event: Event, _gui: &mut Gui) -> Result<(), Error> {
    MSG::post_quit(0);
    Ok(())
}
