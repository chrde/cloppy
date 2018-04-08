use gui::msg::Msg;
use gui::utils::ToWide;
use gui::wnd_proc::wnd_proc;
use parking_lot::Mutex;
use resources::constants::IDR_ACCEL;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::sync::mpsc;
use winapi::shared::minwindef::TRUE;
use winapi::um::winuser::*;
use winapi::um::winuser::WM_APP;
use gui::context_stash::ThreadLocalData;
use gui::context_stash::CONTEXT_STASH;
use winapi::shared::ntdef::LPCWSTR;
use winapi::um::commctrl::*;

mod utils;
mod wnd;
mod wnd_class;
mod msg;
mod context_stash;
mod paint;
mod tray_icon;
mod list_view;
mod input_field;
mod status_bar;
mod wnd_proc;
mod default_font;

type WndId = i32;

const STATUS_BAR_ID: WndId = 1;
const INPUT_SEARCH_ID: WndId = 2;
const FILE_LIST_ID: WndId = 3;
const FILE_LIST_HEADER_ID: WndId = 4;
const MAIN_WND_CLASS: &str = "cloppy_class";
const MAIN_WND_NAME: &str = "Cloppy main window";
const FILE_LIST_NAME: &str = "File list";
const INPUT_TEXT: &str = "Input text";
const STATUS_BAR: &str = "STATUS_BAR";
const INPUT_MARGIN: i32 = 5;
const WM_SYSTRAYICON: u32 = WM_APP + 1;


lazy_static! {
    static ref HASHMAP: Mutex<HashMap<&'static str, Vec<u16>>> = {
        let mut m = HashMap::new();
        m.insert("hello", "hello".to_wide_null());
        m.insert("czesc", "czesc".to_wide_null());
        m.insert("hola", "hola".to_wide_null());
        m.insert("column", "column".to_wide_null());
        m.insert(FILE_LIST_NAME, FILE_LIST_NAME.to_wide_null());
        m.insert(INPUT_TEXT, INPUT_TEXT.to_wide_null());
        m.insert(MAIN_WND_NAME, MAIN_WND_NAME.to_wide_null());
        m.insert(STATUSCLASSNAME, STATUSCLASSNAME.to_wide_null());
        m.insert(STATUS_BAR, STATUS_BAR.to_wide_null());
        m.insert(WC_EDIT, WC_EDIT.to_wide_null());
        m.insert(WC_LISTVIEW, WC_LISTVIEW.to_wide_null());
        Mutex::new(m)
    };
}

fn get_string(str: &str) -> LPCWSTR {
     HASHMAP.lock().get(str).unwrap().as_ptr() as LPCWSTR
}


pub fn init_wingui(sender: mpsc::Sender<OsString>) -> io::Result<i32> {
    let res = unsafe { IsGUIThread(TRUE) };
    assert_ne!(res, 0);
    CONTEXT_STASH.with(|context_stash| {
        *context_stash.borrow_mut() = Some(ThreadLocalData::new(sender, Some(5)));
    });
    wnd_class::WndClass::init_commctrl()?;
    let class = wnd_class::WndClass::new(MAIN_WND_CLASS, wnd_proc)?;
    let accel = match unsafe { LoadAcceleratorsW(class.1, MAKEINTRESOURCEW(IDR_ACCEL)) } {
        v if v.is_null() => utils::other_error("LoadAccelerator failed"),
        v => Ok(v)
    }.unwrap();

    let params = wnd::WndParams::builder()
        .window_name(get_string(MAIN_WND_NAME))
        .class_name(class.0)
        .style(WS_OVERLAPPEDWINDOW)
        .build();
    let wnd = wnd::Wnd::new(params)?;
    wnd.show(SW_SHOWDEFAULT);
    wnd.update()?;
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