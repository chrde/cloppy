use errors::MyErrorKind::WindowsError;
use failure::Error;
use failure::ResultExt;
use gui::utils;
use gui::wnd_class;
use std::{io, mem, ptr};
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::{
    HMENU,
    HWND,
    RECT,
};
use winapi::um::commctrl::GetEffectiveClientRect;
use winapi::um::winuser::*;

#[derive(Copy, Clone)]
pub struct Wnd {
    pub hwnd: HWND,
}

unsafe impl Send for Wnd {}

impl Wnd {
    pub fn new(params: WndParams) -> Result<Self, Error> {
        let instance = match params.instance {
            Some(instance) => instance,
            None => wnd_class::WndClass::get_module_handle()?
        };
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
                v if v.is_null() => utils::last_error().context(WindowsError("CreateWindowExW failed"))?,
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

    pub fn invalidate_rect(&self, rect: Option<RECT>, repaint: bool) {
        let rect = match rect {
            Some(v) => &v as *const _,
            None => ptr::null(),
        };
        unsafe {
            InvalidateRect(self.hwnd, rect as *const _, repaint as i32);
        }
    }

    pub fn window_rect(&self) -> RECT {
        unsafe {
            let mut rect = mem::zeroed::<RECT>();
            GetWindowRect(self.hwnd, &mut rect);
            rect
        }
    }

    pub fn set_as_foreground(&self) -> BOOL {
        unsafe {
            SetForegroundWindow(self.hwnd)
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
    #[default = "ptr::null_mut()"]
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

