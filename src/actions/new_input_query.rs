use dispatcher::GuiDispatcher;
use dispatcher::UiAsyncMessage;
use gui::event::Event;
use gui::HASHMAP;
use std::ffi::OsString;
use winapi::um::winuser::GetWindowTextLengthW;
use winapi::um::winuser::GetWindowTextW;
use windows::utils::FromWide;

pub fn new_input_query(event: Event, dispatcher: &GuiDispatcher) {
    unsafe {
        let length = 1 + GetWindowTextLengthW(event.l_param_mut());
        let mut buffer = vec![0u16; length as usize];
        let read = 1 + GetWindowTextW(event.l_param_mut(), buffer.as_mut_ptr(), length);
        assert_eq!(length, read);
        dispatcher.send_async_msg(UiAsyncMessage::Ui(OsString::from_wide_null(&buffer).to_str().expect("Invalid UI Message").to_string()));
        HASHMAP.lock().insert("hola", buffer);
    }
}