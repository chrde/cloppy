use gui::wnd;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use gui::utils::FromWide;
use std::io;
use gui::context_stash::send_message;
use std::ffi::OsString;
use gui::INPUT_SEARCH_ID;
use gui::HASHMAP;
use gui::get_string;
use gui::INPUT_TEXT;
use gui::wnd_proc::Event;
use Message;

pub fn new(parent: HWND, instance: Option<HINSTANCE>) -> io::Result<wnd::Wnd> {
    let input_params = wnd::WndParams::builder()
        .instance(instance)
        .window_name(get_string(INPUT_TEXT))
        .class_name(get_string(WC_EDIT))
        .h_menu(INPUT_SEARCH_ID as HMENU)
        .style(WS_BORDER | WS_VISIBLE | ES_LEFT | WS_CHILD)
        .h_parent(parent)
        .build();
    wnd::Wnd::new(input_params)
}

pub unsafe fn on_change(event: Event){
    let length = 1 + GetWindowTextLengthW(event.l_param as *mut _);
    let mut buffer = vec![0u16; length as usize];
    let read = 1 + GetWindowTextW(event.l_param as *mut _, buffer.as_mut_ptr(), length);
    assert_eq!(length, read);
    send_message(Message::MSG(OsString::from_wide_null(&buffer)));
    HASHMAP.lock().insert("hola", buffer);
}