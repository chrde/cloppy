use errors::MyErrorKind::WindowsError;
use failure::Error;
use failure::ResultExt;
use gui::get_string;
use gui::set_string;
use gui::STATUS_BAR;
use gui::STATUS_BAR_CONTENT;
use gui::STATUS_BAR_ID;
use gui::wnd::Wnd;
use gui::wnd::WndParams;
use plugin::State;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;


pub fn new(parent: HWND, instance: Option<HINSTANCE>) -> Result<Wnd, Error> {
    let status_bar_params = WndParams::builder()
        .instance(instance)
        .window_name(get_string(STATUS_BAR))
        .h_menu(STATUS_BAR_ID as HMENU)
        .class_name(get_string(STATUSCLASSNAME))
        .h_parent(parent)
        .style(WS_VISIBLE | SBARS_SIZEGRIP | WS_CHILD)
        .build();
    Ok(Wnd::new(status_bar_params).context(WindowsError("Failed to create wnd status_bar"))?)
}

pub struct StatusBar {
    wnd: Wnd,
}

impl StatusBar {
    pub fn new(wnd: Wnd) -> StatusBar {
        StatusBar { wnd }
    }

    pub fn wnd(&self) -> &Wnd {
        &self.wnd
    }

    pub fn update(&self, state: &State) {
        let msg = state.count().to_string() + " objects found";
        set_string(STATUS_BAR_CONTENT, msg.to_string());
        let w_param = (SB_SIMPLEID & (0 << 8)) as WPARAM;
        self.wnd.send_message(SB_SETTEXTW, w_param, get_string(STATUS_BAR_CONTENT) as LPARAM);
    }
}