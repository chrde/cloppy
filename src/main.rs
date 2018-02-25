extern crate winapi;
extern crate conv;
#[macro_use]
extern crate typed_builder;
#[macro_use]
extern crate bitflags;

use gui::msg::Msg;
use gui::wnd;
use gui::wnd_class;
use gui::utils;
use gui::paint;
use std::io;
use std::ptr;
use winapi::um::winuser::{
    DefWindowProcW,
    SendMessageW,
    WM_DESTROY,
    WM_CREATE,
    WM_PAINT,
    WM_RBUTTONUP,
};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::shared::ntdef::LPCWSTR;
use winapi::um::winuser::{
    MSG, WM_QUIT,
};
use winapi::um::commctrl::{
    InitCommonControls,
    CreateStatusWindowW,
};
use gui::utils::ToWide;

mod gui;

const STATUS_BAR: u32 = 123;
const MAIN_WND_CLASS: &str = "hello";
const MAIN_WND_NAME: &str = "hello";

fn main() {
    unsafe {
        InitCommonControls();
    }
    match try_main() {
        Ok(()) => (),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn try_main() -> io::Result<()> {
    let class = wnd_class::WndClass::new(MAIN_WND_CLASS, wnd_proc)?;
    let params = wnd::WndParams::builder()
        .window_name(MAIN_WND_NAME)
        .class_name(class.0)
        .instance(class.1)
        .style(wnd::WndStyle::WS_VISIBLE | wnd::WndStyle::WS_OVERLAPPEDWINDOW)
        .build();
    let mut wnd = wnd::Wnd::new(params)?;
    wnd.show(winapi::um::winuser::SW_SHOWDEFAULT);
    wnd.update()?;
    loop {
        match MSG::get(None).unwrap() {
            MSG { message: WM_QUIT, wParam: code, .. } => {
                ::std::process::exit(code as i32);
            }
            msg => {
                msg.translate();
                msg.dispatch();
            }
        }
    }
    wnd.close();
}

fn status_bar(wnd: HWND) {
    unsafe {
        let style = ::winapi::um::winuser::WS_VISIBLE|::winapi::um::winuser::WS_CHILD|::winapi::um::commctrl::SBARS_SIZEGRIP;
        CreateStatusWindowW(style as i32, "main_status".to_wide_null().as_ptr() as LPCWSTR, wnd, STATUS_BAR);
    }
}

fn status_bar1(wnd: HWND) -> gui::wnd::Wnd{
    let params = wnd::WndParams::builder()
        .class_name("STATUSCLASSNAME".to_wide_null().as_ptr() as LPCWSTR)
        .window_name("main_status_bar")
        .instance(ptr::null_mut())
//        .instance(wnd_class::WndClass::current_instance(wnd).unwrap())
        .style(wnd::WndStyle::WS_VISIBLE | wnd::WndStyle::SBARS_SIZEGRIP | wnd::WndStyle::WS_CHILD)
        .h_parent(wnd)
        .build();
    let wnd = wnd::Wnd::new(params).unwrap();
//    SendMessageW(wnd, )
//    wnd.show(winapi::um::winuser::SW_SHOW);
    wnd
}

unsafe extern "system" fn wnd_proc(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match message {
        WM_DESTROY => {
            MSG::post_quit(0);
            0
        }
        WM_CREATE => {
            status_bar(wnd);
            0
        }
        WM_PAINT => {
            let paint = paint::WindowPaint::new(wnd).unwrap();
            paint.text("Hello world", utils::Location { x: 10, y: 10 }).unwrap();
            0
        }
        WM_RBUTTONUP => {
            println!("holaa");
            0
        }
        message => DefWindowProcW(wnd, message, w_param, l_param),
    }
}
