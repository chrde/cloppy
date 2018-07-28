use errors::MyErrorKind::WindowsError;
use failure::Error;
use failure::ResultExt;
use gui::get_string;
use gui::INPUT_SEARCH_ID;
use gui::wnd;
use gui::Wnd;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;

pub fn new(parent: Wnd, instance: Option<HINSTANCE>) -> Result<Wnd, Error> {
    let input_params = wnd::WndParams::builder()
        .instance(instance)
        .class_name(get_string(WC_EDIT))
        .h_menu(INPUT_SEARCH_ID as HMENU)
        .style(WS_BORDER | WS_VISIBLE | ES_LEFT | WS_CHILD)
        .h_parent(parent.hwnd)
        .build();
    Ok(Wnd::new(input_params).context(WindowsError("Failed to create wnd input_field"))?)
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