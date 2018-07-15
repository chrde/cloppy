use errors::MyErrorKind::WindowsError;
use failure::{Error, ResultExt};
use std::mem;
use std::ptr;
use super::utils;
use super::wnd::Wnd;
use winapi::shared::minwindef::{
    INT,
    LRESULT,
};
use winapi::shared::windef::{
    HACCEL,
    HWND,
};
use winapi::um::winuser::{
    DispatchMessageW,
    GetMessageW,
    MSG,
    PostQuitMessage,
    TranslateAcceleratorW,
    TranslateMessage,
};

pub trait Msg: Sized {
    fn get(wnd: Option<&Wnd>) -> Result<Self, Error>;
    fn dispatch(&self) -> LRESULT;
    fn translate(&self) -> bool;
    fn translate_accel(&mut self, wnd: HWND, accel: HACCEL) -> bool;
    fn post_quit(exit_code: INT);
}

impl Msg for MSG {
    fn get(wnd: Option<&Wnd>) -> Result<MSG, Error> {
        unsafe {
            let wnd = wnd.map_or(ptr::null_mut(), |h| h.hwnd);
            let mut msg = mem::zeroed();
            match GetMessageW(&mut msg, wnd, 0, 0) {
                -1 => utils::last_error().context(WindowsError("GetMessageW failed"))?,
                _ => Ok(msg)
            }
        }
    }
    fn dispatch(&self) -> LRESULT {
        unsafe {
            DispatchMessageW(self)
        }
    }

    fn translate(&self) -> bool {
        unsafe {
            match TranslateMessage(self) {
                0 => false,
                _ => true
            }
        }
    }

    fn translate_accel(&mut self, wnd: HWND, accel: HACCEL) -> bool {
        unsafe {
            match TranslateAcceleratorW(wnd, accel, self as *mut _) {
                0 => false,
                _ => true,
            }
        }
    }

    fn post_quit(exit_code: INT) {
        unsafe {
            PostQuitMessage(exit_code)
        }
    }
}