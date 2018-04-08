use gui::context_stash::send_message;
use gui::utils::ToWide;
use gui::wnd;
use std::io;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winnt::*;
use winapi::um::winuser::*;
use gui::STATUS_BAR_ID;

pub fn new(parent: HWND) -> io::Result<wnd::Wnd> {
    let status_bar_params = wnd::WndParams::builder()
        .window_name("mystatusbar")
        .h_menu(STATUS_BAR_ID as HMENU)
        .class_name(STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR)
        .h_parent(parent)
        .style(WS_VISIBLE | SBARS_SIZEGRIP | WS_CHILD)
        .build();
    wnd::Wnd::new(status_bar_params)
}

pub unsafe fn on_size(_wnd: HWND, _height: i32, _width: i32) {
    send_message(STATUS_BAR_ID, |ref wnd| {
        SendMessageW(wnd.hwnd, WM_SIZE, 0, 0);
    });
}
