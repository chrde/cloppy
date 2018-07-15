use errors::MyErrorKind::WindowsError;
use failure::Error;
use failure::ResultExt;
use gui::utils;
use resources::constants::IDC_CLOPPY;
use std::{mem, ptr};
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

pub type WndProcRef = unsafe extern "system" fn(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;

pub struct WndClass(pub LPCWSTR, pub HINSTANCE);

impl WndClass {
    pub fn new(class_name: LPCWSTR, wnd_proc: WndProcRef) -> Result<Self, Error> {
        unsafe {
            let class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_DBLCLKS,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: WndClass::get_module_handle()?,
                hIcon: ptr::null_mut(),
                hCursor: ptr::null_mut(),
                hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                lpszMenuName: MAKEINTRESOURCEW(IDC_CLOPPY),
                lpszClassName: class_name,
                hIconSm: ptr::null_mut(),
            };
            WndClass::register(&class)
        }
    }

    unsafe fn register(wnd_class: &WNDCLASSEXW) -> Result<Self, Error> {
        match RegisterClassExW(wnd_class) {
            0 => utils::last_error().context(WindowsError("RegisterClassExW failed"))?,
            v => Ok(WndClass(v as LPCWSTR, wnd_class.hInstance))
        }
    }

    pub fn init_commctrl() -> Result<(), Error> {
        unsafe {
            let controls = INITCOMMONCONTROLSEX {
                dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_BAR_CLASSES | ICC_LISTVIEW_CLASSES,
            };
            match InitCommonControlsEx(&controls) {
                0 => utils::last_error().context(WindowsError("InitCommonControlsEx failed"))?,
                _ => Ok(())
            }
        }
    }

    pub fn get_module_handle() -> Result<HMODULE, Error> {
        unsafe {
            match GetModuleHandleW(ptr::null()) {
                v if v.is_null() => utils::last_error().context(WindowsError("GetModuleHandleW failed"))?,
                v => Ok(v)
            }
        }
    }
}
