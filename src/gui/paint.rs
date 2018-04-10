use winapi::um::winuser::{
    BeginPaint,
    EndPaint,
};
use  winapi::um::wingdi::TextOutW;
use std::io;
use std::mem;
use gui::utils::ToWide;
use winapi::um::winuser::PAINTSTRUCT;
use winapi::shared::windef::HWND;
use conv::prelude::*;
use winapi::um::winnt::INT;
use gui::utils;

pub struct WindowPaint(HWND, PAINTSTRUCT);

impl WindowPaint {
    pub fn new(wnd: HWND) -> io::Result<Self> {
        unsafe {
            let mut ps = mem::uninitialized();
            match BeginPaint(wnd, &mut ps) {
                v if v.is_null() => utils::last_error(),
                _ => Ok(WindowPaint(wnd, ps))
            }
        }
    }

    pub fn text(&self, text: &str, x: i32, y: i32) -> io::Result<()> {
        unsafe {
            let string = text.to_wide();
            let length = string.len().value_as::<INT>().unwrap_or_saturate();
            match TextOutW(self.1.hdc, x, y, string.as_ptr(), length){
                0 => utils::other_error("TextOutW failed"),
                _ => Ok(())
            }
        }
    }
}

impl Drop for WindowPaint {
    fn drop(&mut self) {
        unsafe {
            EndPaint(self.0, &self.1);
        }
    }
}