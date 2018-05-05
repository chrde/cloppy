use std::{io, ptr, mem};
use winapi::um::winuser::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::{
    HWND,
    HMENU,
    RECT,
};
use winapi::shared::ntdef::LPCWSTR;
use gui::wnd_class;
use gui::utils;
use winapi::um::commctrl::GetEffectiveClientRect;

#[derive(Clone)]
pub struct Wnd {
    pub hwnd: HWND,
}

unsafe impl Send for Wnd {}

impl Wnd {
    pub fn new(params: WndParams) -> io::Result<Self> {
        let instance = params.instance.unwrap_or_else(|| wnd_class::WndClass::get_module_handle().unwrap());
        unsafe {
            match CreateWindowExW(
                params.ex_style,
                params.class_name,
                params.window_name,
                params.style,
                params.x,
                params.y,
                params.width,
                params.height,
                params.h_parent,
                params.h_menu,
                instance,
                params.lp_param,
            ) {
                v if v.is_null() => utils::last_error(),
                v => Ok(Wnd { hwnd: v }),
            }
        }
    }

    pub fn post_message(&self, message: u32, w_param: WPARAM) {
        unsafe {
            PostMessageW(self.hwnd, message, w_param, 0);
        }
    }

    pub fn send_message(&self, message: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        unsafe {
            SendMessageW(self.hwnd, message, w_param, l_param)
        }
    }

    pub fn set_position(&self, x: i32, y: i32, cx: i32, cy: i32, flags: u32) {
        unsafe {
            SetWindowPos(self.hwnd, ptr::null_mut(), x, y, cx, cy, flags);
        }
    }

    pub fn effective_client_rect(&self, info: [i32; 8]) -> RECT {
        unsafe {
            let mut rect = mem::zeroed::<RECT>();
            GetEffectiveClientRect(self.hwnd, &mut rect, info.as_ptr());
            rect
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
pub struct WndParams {
    window_name: LPCWSTR,
    class_name: LPCWSTR,
    #[default = "None"]
    instance: Option<HINSTANCE>,
    style: DWORD,
    #[default = "0"]
    ex_style: DWORD,
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
    #[default = "0"]
    x: INT,
    #[default = "0"]
    y: INT,
}

