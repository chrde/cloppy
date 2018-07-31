use dispatcher::UiAsyncMessage;
use failure::Error;
use gui::event::Event;
use gui::Gui;

pub fn new_input_query(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    let text = gui.input_search().wnd().get_text()?;
    gui.dispatcher().send_async_msg(UiAsyncMessage::Ui(text));
    Ok(())
}