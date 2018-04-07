#![allow(dead_code)]
extern crate conv;
#[macro_use]
extern crate typed_builder;
#[macro_use]
extern crate lazy_static;
extern crate winapi;
extern crate parking_lot;

use gui::msg::Msg;
use gui::tray_icon;
use gui::utils;
use gui::utils::FromWide;
use gui::utils::ToWide;
use gui::wnd;
use gui::list_view;
use gui::wnd_class;
use resources::constants::*;
use std::ffi::OsString;
use std::io;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::TRUE;
use winapi::um::commctrl::*;
use winapi::um::shellapi::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;
use std::collections::HashMap;
use parking_lot::Mutex;
use std::thread;
use std::sync::mpsc;
use context_stash::*;
use gui::input_field;

mod gui;
mod resources;
mod context_stash;

pub type WndId = i32;

const STATUS_BAR_ID: WndId = 1;
const INPUT_SEARCH_ID: WndId = 2;
const FILE_LIST_ID: WndId = 3;
const MAIN_WND_CLASS: &str = "cloppy_class";
const MAIN_WND_NAME: &str = "cloppy_main_window";
const INPUT_MARGIN: i32 = 5;
pub const WM_SYSTRAYICON: u32 = WM_APP + 1;


lazy_static! {
    pub static ref HASHMAP: Mutex<HashMap<i32, Vec<u16>>> = {
        let mut m = HashMap::new();
        m.insert(0, "hello".to_wide_null());
        m.insert(1, "czesc".to_wide_null());
        m.insert(2, "hola".to_wide_null());
        Mutex::new(m)
    };
}

fn main() {
    match try_main() {
        Ok(code) => ::std::process::exit(code),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn default_font() -> Result<HFONT, io::Error> {
    unsafe {
        let mut metrics = mem::zeroed::<NONCLIENTMETRICSW>();
        let size = mem::size_of::<NONCLIENTMETRICSW>() as u32;
        metrics.cbSize = size;
        let font = match SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            size,
            &mut metrics as *mut _ as *mut _,
            0)
            {
                v if v == 0 => utils::last_error(),
                _ => Ok(metrics.lfMessageFont),
            }?;
        match CreateFontIndirectW(&font) {
            v if v.is_null() => utils::other_error("CreateFontIndirectW failed"),
            v => Ok(v)
        }
    }
}

fn try_main() -> io::Result<i32> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let res = unsafe { IsGUIThread(TRUE) };
        assert_ne!(res, 0);
        CONTEXT_STASH.with(|context_stash| {
            *context_stash.borrow_mut() = Some(ThreadLocalData::new(sender, Some(5)));
        });
        init_wingui().unwrap();
    });
    run_forever(receiver);
    Ok(0)
}

fn run_forever(receiver: mpsc::Receiver<OsString>) {
    loop {
        let event = match receiver.recv() {
            Ok(e) => e,
            Err(_) => {
                println!("Channel closed. Probably UI thread exit.");
                return;
            }
        };
        println!("{:?}", event);
    }
}

fn init_wingui() -> io::Result<i32> {
    wnd_class::WndClass::init_commctrl()?;
    let class = wnd_class::WndClass::new(MAIN_WND_CLASS, wnd_proc)?;
    let accel = match unsafe { LoadAcceleratorsW(class.1, MAKEINTRESOURCEW(101)) } {
        v if v.is_null() => utils::other_error("LoadAccelerator failed"),
        v => Ok(v)
    }.unwrap();

    let params = wnd::WndParams::builder()
        .window_name(MAIN_WND_NAME)
        .class_name(class.0)
        .style(WS_OVERLAPPEDWINDOW)// | WS_CLIPCHILDREN)
        .build();
    let wnd = wnd::Wnd::new(params)?;
//    main_menu(wnd.hwnd)?;
    let status_bar_params = wnd::WndParams::builder()
        .window_name("mystatusbar")
        .h_menu(STATUS_BAR_ID as HMENU)
        .class_name(STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR)
        .h_parent(wnd.hwnd)
        .style(WS_VISIBLE | SBARS_SIZEGRIP | WS_CHILD)
        .build();
    wnd::Wnd::new(status_bar_params)?;
    wnd.show(winapi::um::winuser::SW_SHOWDEFAULT);
    wnd.update()?;
    unsafe { EnumChildWindows(wnd.hwnd, Some(font_proc), default_font().unwrap() as LPARAM); }
    let mut icon = tray_icon::TrayIcon::new(&wnd);
    icon.set_visible()?;
    loop {
        match MSG::get(None).unwrap() {
            MSG { message: WM_QUIT, wParam: code, .. } => {
                return Ok(code as i32);
            }
            mut msg => {
                if !msg.translate_accel(wnd.hwnd, accel) {
                    msg.translate();
                    msg.dispatch();
                }
            }
        }
    }
}

unsafe extern "system" fn font_proc(wnd: HWND, font: LPARAM) -> BOOL {
    SendMessageW(wnd, WM_SETFONT, font as WPARAM, TRUE as LPARAM);
    TRUE
}

unsafe extern "system" fn wnd_proc(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match message {
        WM_DESTROY => {
            MSG::post_quit(0);
            0
        }
        WM_CREATE => {
            add_window(FILE_LIST_ID, list_view::new(wnd).unwrap());
            add_window(INPUT_SEARCH_ID, input_field::new(wnd).unwrap());
            0
        }
        WM_NOTIFY => {
            match (*(l_param as LPNMHDR)).code {
                LVN_GETDISPINFOW => {
                    let mut plvdi = *(l_param as LPNMLVDISPINFOW);
                    if (plvdi.item.mask & LVIF_TEXT) == LVIF_TEXT {
                        (*(l_param as LPNMLVDISPINFOW)).item.pszText = HASHMAP.lock().get(&plvdi.item.iSubItem).unwrap().as_ptr() as LPWSTR;
//                        match plvdi.item.iSubItem {
//                            0 => {
//                                (*(l_param as LPNMLVDISPINFOW)).item.pszText = HASHMAP.get(&0).unwrap().as_ptr() as LPWSTR;
//                            }
//                            2 => {
//                                println!("asking for {} {}", plvdi.item.iItem, plvdi.item.iSubItem);
//                                plvdi.item.pszText = "column 2".to_wide_null().as_ptr() as LPWSTR;
//                            }
//                            _ => {
//                                println!("WTF");
//                                unreachable!();
//                            }
//                        }
                    }
                    1
                }
                _ => {
                    DefWindowProcW(wnd, message, w_param, l_param)
                }
            }
        }
        WM_SIZE => {
            let new_width = LOWORD(l_param as u32) as i32;
            let new_height = HIWORD(l_param as u32) as i32;

            input_field::on_size(wnd, new_height, new_width);

            let status_bar = GetDlgItem(wnd, STATUS_BAR_ID);
            SendMessageW(status_bar, WM_SIZE, 0, 0);

            list_view::on_size(wnd, new_height, new_width);
            0
//            DefWindowProcW(wnd, message, w_param, l_param)
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
        WM_COMMAND => {
            match HIWORD(w_param as u32) as u16 {
                EN_CHANGE => {
                    input_field::on_change(wnd, w_param, l_param);
                    InvalidateRect(wnd, ptr::null_mut(), 0);
                }
                _ => {
                    match LOWORD(w_param as u32) as u32 {
                        ID_FILL_LIST => {
                            let list_view = GetDlgItem(wnd, FILE_LIST_ID);
                            SendMessageW(list_view, LVM_SETITEMCOUNT, 2000000, 0);
                        }
                        ID_SELECT_ALL => {
                            let focused_wnd = GetFocus();
                            if !focused_wnd.is_null() {
                                let mut buffer = [0u16; 20];
                                let bytes_read = GetClassNameW(focused_wnd, buffer.as_mut_ptr(), buffer.len() as i32);
                                if bytes_read != 0 {
                                    let class = OsString::from_wide_null(&buffer);
                                    match class.to_string_lossy().as_ref() {
                                        WC_EDIT => {
                                            SendMessageW(focused_wnd, EM_SETSEL as u32, 0, -1);
                                        }
                                        _ => {
                                            println!("todo");
                                        }
                                    }
                                }
                            }
                            println!("{:?}", wnd);
                            let input_text = FindWindowExW(wnd, ptr::null_mut(), WC_EDIT.to_wide_null().as_ptr() as LPCWSTR, ptr::null_mut());
                            SendMessageW(input_text, EM_SETSEL as u32, 0, -1);
                        }
                        _ => {}
                    }
                }
            }
            DefWindowProcW(wnd, message, w_param, l_param)
        }
//        WM_RBUTTONUP => {
//            println!("holaa");
//            0
//        }
        message => DefWindowProcW(wnd, message, w_param, l_param),
    }
}
