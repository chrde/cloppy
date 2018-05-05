use gui::context_stash::apply_on_window;
use gui::get_string;
use gui::STATUS_BAR;
use gui::STATUS_BAR_CONTENT;
use gui::STATUS_BAR_ID;
use gui::wnd;
use std::io;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use gui::set_string;
use gui::context_stash::CONTEXT_STASH;


pub fn new(parent: HWND, instance: Option<HINSTANCE>) -> io::Result<wnd::Wnd> {
    let status_bar_params = wnd::WndParams::builder()
        .instance(instance)
        .window_name(get_string(STATUS_BAR))
        .h_menu(STATUS_BAR_ID as HMENU)
        .class_name(get_string(STATUSCLASSNAME))
        .h_parent(parent)
        .style(WS_VISIBLE | SBARS_SIZEGRIP | WS_CHILD)
        .build();
    wnd::Wnd::new(status_bar_params)
}

pub fn update_status_bar() {
    let size = CONTEXT_STASH.with(|context_stash| {
        let mut context_stash = context_stash.borrow_mut();
        context_stash.as_mut().unwrap().state.count()
    });
    let status_bar_message = size.to_string() + " objects found";
    set_string(STATUS_BAR_CONTENT, status_bar_message);
    apply_on_window(STATUS_BAR_ID, |ref wnd| {
        //SBT_NOBORDERS
        let w_param = (SB_SIMPLEID & (0 << 8)) as WPARAM;
        unsafe { SendMessageW(wnd.hwnd, SB_SETTEXTW, w_param, get_string(STATUS_BAR_CONTENT) as LPARAM); }
    });
}