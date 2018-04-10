use gui::wnd;
use std::io;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use gui::STATUS_BAR_ID;
use gui::get_string;
use gui::STATUS_BAR;

pub fn new(parent: HWND) -> io::Result<wnd::Wnd> {
    let status_bar_params = wnd::WndParams::builder()
        .window_name(get_string(STATUS_BAR))
        .h_menu(STATUS_BAR_ID as HMENU)
        .class_name(get_string(STATUSCLASSNAME))
        .h_parent(parent)
        .style(WS_VISIBLE | SBARS_SIZEGRIP | WS_CHILD)
        .build();
    wnd::Wnd::new(status_bar_params)
}
