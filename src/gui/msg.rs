use std::io;
use super::wnd::Wnd;
use super::utils;
use winapi::shared::minwindef::{
    LRESULT,
    INT
};
use winapi::um::winuser::{
    GetMessageW,
    MSG,
    TranslateMessage,
    DispatchMessageW,
    PostQuitMessage,
};
use std::mem;
use std::ptr;

pub trait Msg: Sized {
    fn get(wnd: Option<&Wnd>) -> io::Result<Self>;
    fn dispatch(&self) -> LRESULT;
    fn translate(&self) -> bool;
    fn post_quit(exit_code: INT);
}

impl Msg for MSG {
    fn get(wnd: Option<&Wnd>) -> io::Result<Self> {
        unsafe {
            let wnd = wnd.map_or(ptr::null_mut(), |h| h.hwnd);
            let mut msg = mem::zeroed();
            match GetMessageW(&mut msg, wnd, 0, 0) {
                -1 => utils::last_error(),
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

    fn post_quit(exit_code: INT) {
        unsafe {
            PostQuitMessage(exit_code)
        }
    }
}