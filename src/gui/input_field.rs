use gui::wnd;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use winapi::um::winnt::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use gui::utils::Location;
use gui::utils::FromWide;
use gui::utils::ToWide;
use INPUT_SEARCH_ID;
use INPUT_MARGIN;
use HASHMAP;
use std::io;
use context_stash::send_message;
use std::ptr;
use context_stash::send_event;
use std::ffi::OsString;

pub fn new(parent: HWND) -> io::Result<wnd::Wnd> {
    let input_params = wnd::WndParams::builder()
        .window_name("myinputtext")
        .class_name(WC_EDIT.to_wide_null().as_ptr() as LPCWSTR)
        .h_menu(INPUT_SEARCH_ID as HMENU)
        .style(WS_BORDER | WS_VISIBLE | ES_LEFT | WS_CHILD)
        .h_parent(parent)
        .location(Location { x: INPUT_MARGIN, y: INPUT_MARGIN })
        .build();
    wnd::Wnd::new(input_params)
}

pub unsafe fn on_size(_wnd: HWND, _height: i32, width: i32) {
    send_message(INPUT_SEARCH_ID, |ref wnd| {
        SetWindowPos(wnd.hwnd, ptr::null_mut(), 0, 0, width - 2 * INPUT_MARGIN, 20, SWP_NOMOVE);
    });
}

pub unsafe fn on_change(_wnd: HWND, _w_param: WPARAM, l_param: LPARAM){
    let length = 1 + GetWindowTextLengthW(l_param as *mut _);
    let mut buffer = vec![0u16; length as usize];
    let read = 1 + GetWindowTextW(l_param as *mut _, buffer.as_mut_ptr(), length);
    assert_eq!(length, read);
    send_event(OsString::from_wide_null(&buffer));
    HASHMAP.lock().insert(0, buffer);
}