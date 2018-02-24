use super::wnd_class::WndClass;
use super::utils;
use super::utils::ToWide;
use std::{io, ptr};
use winapi::um::winuser::{
    CreateWindowExW, DestroyWindow, WS_OVERLAPPEDWINDOW, WS_VISIBLE, CW_USEDEFAULT,
};
use winapi::shared::windef::HWND;
use winapi::shared::ntdef::LPCWSTR;

pub struct Wnd(pub HWND);

impl Wnd {
    pub fn new(window_name: &str, class: &WndClass) -> io::Result<Self> {
        let &WndClass(class_name, instance) = class;
        unsafe {
            match CreateWindowExW(
                0,
                class_name as LPCWSTR,
                window_name.to_wide_null().as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            ) {
                v if v.is_null() => utils::last_error(),
                v => Ok(Wnd(v))
            }
        }
    }
}

impl Drop for Wnd {
    fn drop(&mut self) {
        unsafe {
            let result = match DestroyWindow(self.0) {
                0 => utils::last_error(),
                _ => Ok(())
            };
            result.unwrap()
        }
    }
}
