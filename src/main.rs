#![allow(dead_code)]
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
use gui::tray_icon;
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
    WM_APP,
    WM_PAINT,
    WM_SIZE,
    WM_LBUTTONUP,
    WM_LBUTTONDBLCLK,
};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{
    HWND,
};
use winapi::um::shellapi::{
    NIN_KEYSELECT,
    NIN_SELECT,
};
use winapi::shared::ntdef::LPCWSTR;
use winapi::um::winuser::{
    MSG, WM_QUIT,
};
use winapi::um::commctrl::STATUSCLASSNAME;
use gui::utils::ToWide;

mod gui;
mod resources;

const STATUS_BAR: u32 = 123;
const MAIN_WND_CLASS: &str = "hello";
const MAIN_WND_NAME: &str = "hello";
pub const WM_SYSTRAYICON: u32 = WM_APP + 1;

fn main() {
    match try_main() {
        Ok(code) => ::std::process::exit(code),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn try_main() -> io::Result<i32> {
    wnd_class::WndClass::init_commctrl()?;
    let class = wnd_class::WndClass::new(MAIN_WND_CLASS, wnd_proc)?;
    let params = wnd::WndParams::builder()
        .window_name(MAIN_WND_NAME)
        .class_name(class.0)
        .instance(class.1)
        .style(wnd::WndStyle::WS_OVERLAPPEDWINDOW)
        .build();
    let wnd = wnd::Wnd::new(params)?;
    wnd.show(winapi::um::winuser::SW_SHOWDEFAULT);
    main_menu(wnd.hwnd)?;
    let status_bar_params = wnd::WndParams::builder()
        .window_name("mystatusbar")
        .class_name(STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR)
        .instance(class.1)
        .h_parent(wnd.hwnd)
        .style(wnd::WndStyle::WS_VISIBLE | wnd::WndStyle::SBARS_SIZEGRIP | wnd::WndStyle::WS_CHILD)
        .build();
    wnd::Wnd::new(status_bar_params)?;
    wnd.update()?;
    let mut icon = tray_icon::TrayIcon::new(wnd);
    icon.set_visible()?;
    loop {
        match MSG::get(None).unwrap() {
            MSG { message: WM_QUIT, wParam: code, .. } => {
                return Ok(code as i32);
            }
            msg => {
                msg.translate();
                msg.dispatch();
            }
        }
    }
}

fn main_menu(wnd: HWND) -> io::Result<()> {
    unsafe {
        let result = match CreateMenu() {
            v if v.is_null() => utils::last_error(),
            v => Ok(v)
        };
        let menu = result?;
        let x: MENUITEMINFOW = MENUITEMINFOW {
            cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_ID | MIIM_STRING | MIIM_DATA | MIIM_FTYPE,
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
        let result = match InsertMenuItemW(menu, 0, 1, &x) {
            0 => utils::last_error(),
            _ => Ok(())
        };
        let _ = result?;
        match SetMenu(wnd, menu) {
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
            let status_bar = FindWindowExW(wnd, ptr::null_mut(), STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR, ptr::null_mut());
            SendMessageW(status_bar, WM_SIZE, 0, 0);
            DefWindowProcW(wnd, message, w_param, l_param)
        }
        WM_SYSTRAYICON => {
            match l_param as u32 {
                NIN_KEYSELECT | NIN_SELECT | WM_LBUTTONUP => {
                    println!("selected");
                }
                WM_LBUTTONDBLCLK => {
                    println!("double click");
                }
                _ => {}
            };
            0

        }
//        WM_SYSCOMMAND => {
//            println!("{:?}-{:?}-{:?}", message, w_param & 0xFFF0, l_param);
//            0
//
//        }
//        WM_COMMAND => {
//            println!("two");
//            0
//
//        }
//        WM_CREATE => {
//            main_menu(wnd).unwrap();
//            0
//        }
        WM_PAINT => {
            let paint = paint::WindowPaint::new(wnd).unwrap();
            paint.text("Hello world", utils::Location { x: 10, y: 10 }).unwrap();
            0
        }
//        WM_RBUTTONUP => {
//            println!("holaa");
//            0
//        }
        message => DefWindowProcW(wnd, message, w_param, l_param),
    }
}
