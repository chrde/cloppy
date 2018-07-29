use errors::MyErrorKind::WindowsError;
use failure::Error;
use failure::ResultExt;
use gui::utils;
use gui::wnd_class;
use std::{io, mem, ptr};
use std::ffi::OsString;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::{
    HMENU,
    HWND,
    RECT,
};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::commctrl::GetEffectiveClientRect;
use winapi::um::winuser::*;
use windows::utils::FromWide;

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

    pub fn post_message(&self, message: u32, w_param: WPARAM, l_param: LPARAM) {
        unsafe {
            PostMessageW(self.hwnd, message, w_param, l_param);
        }
    }

    pub fn send_message(&self, message: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        unsafe {
            SendMessageW(self.hwnd, message, w_param, l_param)
        }
    }

    pub fn set_position(&self, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> Result<(), Error> {
        match unsafe {
            SetWindowPos(self.hwnd, ptr::null_mut(), x, y, cx, cy, flags)
        } {
            0 => Err(io::Error::last_os_error()).context(WindowsError("SetWindowPos failed"))?,
            _ => Ok(())
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

    pub fn window_rect(&self) -> Result<RECT, Error> {
        unsafe {
            let mut rect = mem::zeroed::<RECT>();
            match GetWindowRect(self.hwnd, &mut rect) {
                0 => Err(io::Error::last_os_error()).context(WindowsError("GetWindowRect failed"))?,
                _ => Ok(rect)
            }
        }
    }

    pub fn set_focus(&self) -> Result<(), Error> {
        match unsafe {
            SetFocus(self.hwnd)
        } {
            v if v.is_null() => Err(io::Error::last_os_error()).context(WindowsError("SetFocus failed"))?,
            _ => Ok(())
        }
    }

    pub fn set_as_foreground(&self) -> Result<(), Error> {
        match unsafe {
            SetForegroundWindow(self.hwnd)
        } {
            0 => Err(io::Error::last_os_error()).context(WindowsError("SetForegroundWindow failed"))?,
            _ => Ok(())
        }
    }

    pub fn effective_client_rect(&self, info: [i32; 8]) -> RECT {
        unsafe {
            let mut rect = mem::zeroed::<RECT>();
            GetEffectiveClientRect(self.hwnd, &mut rect, info.as_ptr());
            rect
        }
    }

    pub fn show(&self, mode: INT) -> Result<(), Error> {
        match unsafe {
            ShowWindow(self.hwnd, mode)
        } {
            0 => match io::Error::last_os_error() {
                ref e if e.raw_os_error() == Some(ERROR_SUCCESS as i32) => Ok(()),
                e => Err(e).context(WindowsError("ShowWindow failed"))?,
            },
            _ => Ok(())
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

    pub fn get_text_length(&self) -> Result<i32, Error> {
        //empty string length = 1 -> it includes the \0 terminator
        match unsafe {
            GetWindowTextLengthW(self.hwnd)
        } {
            0 => match io::Error::last_os_error() {
                ref e if e.raw_os_error() == Some(ERROR_SUCCESS as i32) => Ok(1),
                e => Err(e).context(WindowsError("GetWindowTextLengthW failed"))?,
            },
            v => Ok(v + 1)
        }
    }

    pub fn get_text(&self) -> Result<String, Error> {
        //empty string length = 1 -> it includes the \0 terminator
        let mut buffer = vec![0u16; self.get_text_length()? as usize];
        let read: Result<i32, Error> = match unsafe {
            GetWindowTextW(self.hwnd, buffer.as_mut_ptr(), buffer.len() as i32)
        } {
            0 => match io::Error::last_os_error() {
                ref e if e.raw_os_error() == Some(ERROR_SUCCESS as i32) => Ok(1),
                e => Err(e).context(WindowsError("GetWindowTextW failed"))?,
            },
            v => Ok(v + 1)
        };
        assert_eq!(buffer.len() as i32, read?);
        OsString::from_wide_null(&buffer)
            .to_str()
            .map(|s| s.to_string())
            .ok_or(WindowsError("ShowWindow failed").into())
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

