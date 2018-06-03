use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HWND;
use winapi::um::commctrl::*;
use winapi::um::winuser::LPDRAWITEMSTRUCT;
use winapi::um::winuser::DRAWITEMSTRUCT;

#[derive(Copy, Clone)]
pub struct Event {
    wnd: HWND,
    l_param: LPARAM,
    w_param: WPARAM,
}

impl Event {
    pub fn new(wnd: HWND, l_param: LPARAM, w_param: WPARAM) -> Event {
        Event { wnd, l_param, w_param }
    }

    pub fn wnd(&self) -> HWND {
        self.wnd
    }

    pub fn l_param(&self) -> LPARAM {
        self.l_param
    }

    pub fn w_param(&self) -> WPARAM {
        self.w_param
    }

    pub fn as_cache_hint(&self) -> &mut NMLVCACHEHINT {
        unsafe { &mut *(self.l_param as LPNMLVCACHEHINT) }
    }

    pub fn as_display_info(&self) -> &mut NMLVDISPINFOW {
        unsafe { &mut *(self.l_param as LPNMLVDISPINFOW) }
    }

    pub fn as_list_view(&self) -> &mut NMLISTVIEW {
        unsafe { &mut *(self.l_param as LPNMLISTVIEW) }
    }

    pub fn as_list_header_change(&self) -> &mut NMHEADERW {
        unsafe { &mut *(self.l_param as LPNMHEADERW) }
    }

    pub fn as_draw_item(&self) -> &mut DRAWITEMSTRUCT {
        unsafe { &mut *(self.l_param as LPDRAWITEMSTRUCT) }
    }

    pub fn l_param_mut<T>(&self) -> *mut T {
        self.l_param as *mut T
    }

    pub fn w_param_mut<T>(&self) -> *mut T {
        self.w_param as *mut T
    }
}

