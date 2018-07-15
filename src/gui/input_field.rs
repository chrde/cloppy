use dispatcher::GuiDispatcher;
use dispatcher::UiAsyncMessage;
use errors::MyErrorKind::WindowsError;
use failure::Error;
use failure::ResultExt;
use gui::event::Event;
use gui::get_string;
use gui::HASHMAP;
use gui::INPUT_SEARCH_ID;
use gui::utils::FromWide;
use gui::wnd;
use gui::Wnd;
use std::ffi::OsString;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;

pub fn new(parent: HWND, instance: Option<HINSTANCE>) -> Result<Wnd, Error> {
    let input_params = wnd::WndParams::builder()
        .instance(instance)
        .class_name(get_string(WC_EDIT))
        .h_menu(INPUT_SEARCH_ID as HMENU)
        .style(WS_BORDER | WS_VISIBLE | ES_LEFT | WS_CHILD)
        .h_parent(parent)
        .build();
    Ok(wnd::Wnd::new(input_params).context(WindowsError("Failed to create wnd input_field"))?)
}

pub unsafe fn on_change(event: Event, dispatcher: &GuiDispatcher) {
    let length = 1 + GetWindowTextLengthW(event.l_param_mut());
    let mut buffer = vec![0u16; length as usize];
    let read = 1 + GetWindowTextW(event.l_param_mut(), buffer.as_mut_ptr(), length);
    assert_eq!(length, read);
    dispatcher.send_async_msg(UiAsyncMessage::Ui(OsString::from_wide_null(&buffer).to_str().expect("Invalid UI Message").to_string()));
    HASHMAP.lock().insert("hola", buffer);
}

pub struct InputSearch {
    wnd: Wnd,
}

impl InputSearch {
    pub fn new(wnd: Wnd) -> InputSearch {
        InputSearch { wnd }
    }

    pub fn wnd(&self) -> &Wnd {
        &self.wnd
    }
}