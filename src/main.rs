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
use std::mem;
use std::ptr;
use winapi::um::winuser::{
    DefWindowProcW,
    SendMessageW,
    FindWindowExW,
    CreateMenu,
    InsertMenuItemW,
    SetMenu,
    MIIM_STRING,
    MIIM_ID,
    MIIM_DATA,
    MIIM_FTYPE,
    MFT_STRING,
    MFS_ENABLED,
    MENUITEMINFOW,
    WM_DESTROY,
    WM_CREATE,
    WM_PAINT,
    WM_COMMAND,
    WM_SYSCOMMAND,
    WM_SIZE,
    WM_RBUTTONUP,
};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{
    HWND,
    HMENU,
};
use winapi::shared::ntdef::LPCWSTR;
use winapi::um::winuser::{
    MSG, WM_QUIT,
};
use winapi::um::commctrl::{
    InitCommonControlsEx,
    INITCOMMONCONTROLSEX,
    ICC_BAR_CLASSES,
    CreateStatusWindowW,
    STATUSCLASSNAME,
};
use gui::utils::ToWide;

mod gui;

const STATUS_BAR: u32 = 123;
const MAIN_WND_CLASS: &str = "hello";
const MAIN_WND_NAME: &str = "hello";

fn main() {
    unsafe {
        let controls = INITCOMMONCONTROLSEX {
            dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC : ICC_BAR_CLASSES,
        };
        InitCommonControlsEx(&controls);
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
        .style(wnd::WndStyle::WS_OVERLAPPEDWINDOW)
        .build();
    let wnd = wnd::Wnd::new(params).unwrap();
    wnd.show(winapi::um::winuser::SW_SHOWDEFAULT);
    main_menu(wnd.hwnd)?;
    let status_bar_params = wnd::WndParams::builder()
        .window_name("main_status_bar")
        .class_name(STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR)
        .instance(class.1)
        .h_parent(wnd.hwnd)
        .style(wnd::WndStyle::WS_VISIBLE | wnd::WndStyle::SBARS_SIZEGRIP | wnd::WndStyle::WS_CHILD)
        .build();
    let status_bar = wnd::Wnd::new(status_bar_params)?;
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
}

fn main_menu(wnd: HWND) -> io::Result<()>{
    unsafe {
        let result = match CreateMenu()  {
            v if v.is_null() => utils::last_error(),
            v => Ok(v)
        };
        let menu = result?;
        let x : MENUITEMINFOW = MENUITEMINFOW {
            cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_ID | MIIM_STRING | MIIM_DATA | MIIM_FTYPE ,
            fType: MFT_STRING,
            fState: MFS_ENABLED,
            wID: 1,
            hSubMenu: ptr::null_mut(),
            hbmpChecked: ptr::null_mut(),
            hbmpUnchecked: ptr::null_mut(),
            dwItemData: 0,
            dwTypeData: "&File".to_wide_null().as_mut_ptr(),
            cch: "File".len() as u32,
            hbmpItem: ptr::null_mut(),
        };
        let result = match InsertMenuItemW(menu, 0, 1, &x)  {
            0 => utils::last_error(),
            _ => Ok(())
        };
        let _ = result?;
        match SetMenu(wnd, menu)  {
            0 => utils::last_error(),
            _ => Ok(())
        }
    }
}

unsafe extern "system" fn wnd_proc(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match message {
        WM_DESTROY => {
            MSG::post_quit(0);
            0
        }
        WM_SIZE => {
                let status_bar = FindWindowExW(wnd, ptr::null_mut(),STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR, ptr::null_mut());
                SendMessageW(status_bar, WM_SIZE,0, 0);
            0
        }
        WM_SYSCOMMAND => {
            println!("{:?}-{:?}-{:?}", message, w_param & 0xFFF0, l_param);
            0

        }
        WM_COMMAND => {
            println!("two");
            0

        }
//        WM_CREATE => {
//            main_menu(wnd).unwrap();
//            0
//        }
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
