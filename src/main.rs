extern crate winapi;

use gui::msg::Msg;
use std::io;
use winapi::um::winuser::{
    DefWindowProcW, WM_DESTROY,
};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{
    MSG, WM_QUIT,
};

mod gui;

fn main() {
    match try_main() {
        Ok(()) => (),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn try_main() -> io::Result<()> {
    let class = gui::wnd_class::WndClass::new("hello", wnd_proc)?;
    let w = gui::wnd::Wnd::new("hello", &class)?;
    loop {
        match MSG::get(None)? {
            MSG { message: WM_QUIT, wParam: code, .. } => {
                ::std::process::exit(code as i32);
            }
            msg => {
                msg.translate();
                msg.dispatch();
            }
        }
    }
}

unsafe extern "system" fn wnd_proc(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match message {
        WM_DESTROY => {
            MSG::post_quit(0);
            0
        }
        message => DefWindowProcW(wnd, message, w_param, l_param),
    }
}
