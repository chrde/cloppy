use gui::utils;
use gui::utils::ToWide;
use std::{io, ptr};
use winapi::um::winuser::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::{
    HWND,
    HMENU,
};
use winapi::shared::ntdef::LPCWSTR;

pub struct Wnd {
    pub hwnd: HWND,
}

impl Wnd {
    pub fn new(params: WndParams) -> io::Result<Self> {
        unsafe {
            match CreateWindowExW(
                0,
                params.class_name,
                params.window_name.to_wide_null().as_ptr(),
                params.style.bits,
                params.location.x,
                params.location.y,
                params.width,
                params.height,
                params.h_parent,
                params.h_menu,
                params.instance,
                params.lp_param,
            ) {
                v if v.is_null() => utils::last_error(),
                v => {
                    println!("Created window {:?}{:?}", params.window_name, v);
                    Ok(Wnd { hwnd: v })
                }
            }
        }
    }

    pub fn show(&self, mode: INT) -> BOOL {
        unsafe {
            ShowWindow(self.hwnd, mode)
        }
    }

    pub fn update(&self) -> io::Result<()> {
        unsafe {
            match UpdateWindow(self.hwnd) {
                0 => utils::last_error(),
                _ => Ok(())
            }
        }
    }
}

#[derive(TypedBuilder)]
pub struct WndParams<'a> {
    window_name: &'a str,
    class_name: LPCWSTR,
    instance: HINSTANCE,
    style: WndStyle,
    #[default = "ptr::null_mut()"]
    h_parent: HWND,
    #[default = "ptr::null_mut()"]
    h_menu: HMENU,
    #[default = "ptr::null_mut()"]
    lp_param: LPVOID,
    #[default = "CW_USEDEFAULT"]
    width: INT,
    #[default = "CW_USEDEFAULT"]
    height: INT,
    #[default = "Default::default()"]
    location: utils::Location,
}

bitflags! {
    pub struct WndStyle: DWORD {
        const WS_VISIBLE = ::winapi::um::winuser::WS_VISIBLE;
        const WS_OVERLAPPEDWINDOW = ::winapi::um::winuser::WS_OVERLAPPEDWINDOW;
        const WS_CHILD = ::winapi::um::winuser::WS_CHILD;
        const SBARS_SIZEGRIP  = ::winapi::um::commctrl::SBARS_SIZEGRIP;
        const WS_BORDER = ::winapi::um::winuser::WS_BORDER;
        const ES_LEFT = ::winapi::um::winuser::ES_LEFT;
        const LVS_REPORT = ::winapi::um::commctrl::LVS_REPORT;
        const LVS_SHOWSELALWAYS = ::winapi::um::commctrl::LVS_SHOWSELALWAYS;
        const LVS_ALIGNLEFT = ::winapi::um::commctrl::LVS_ALIGNLEFT;
        const LVS_OWNERDATA = ::winapi::um::commctrl::LVS_OWNERDATA;
        const LVS_ICON = ::winapi::um::commctrl::LVS_ICON;
    }
}
