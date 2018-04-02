use super::utils::ToWide;
use std::{io, mem, ptr};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM, HMODULE};
use winapi::um::winuser::{
    RegisterClassExW,
    UnregisterClassW,
    GetClassInfoExW,
    WNDCLASSEXW,
    GetWindowLongPtrW,
    GWL_HINSTANCE,
    CS_DBLCLKS,
};
use gui::utils;

use winapi::um::commctrl::{
    InitCommonControlsEx,
    INITCOMMONCONTROLSEX,
    ICC_BAR_CLASSES,
    ICC_LISTVIEW_CLASSES,
};

pub type WndProcRef = unsafe extern "system" fn(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;

pub struct WndClass(pub LPCWSTR, pub HINSTANCE);

impl WndClass {
    pub fn new(class_name: &str, wnd_proc: WndProcRef) -> io::Result<Self> {
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
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null_mut(),
                lpszClassName: class_name.to_wide_null().as_ptr(),
                hIconSm: ptr::null_mut(),
            };
            WndClass::register(&class)
        }
    }

    unsafe fn register(wnd_class: &WNDCLASSEXW) -> io::Result<Self> {
        match RegisterClassExW(wnd_class) {
            0 => utils::last_error(),
            v => Ok(WndClass(v as LPCWSTR, wnd_class.hInstance))
        }
    }

    pub fn init_commctrl() -> io::Result<()> {
        unsafe {
            let controls = INITCOMMONCONTROLSEX {
                dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_BAR_CLASSES | ICC_LISTVIEW_CLASSES,
            };
            match InitCommonControlsEx(&controls) {
                0 => utils::last_error(),
                _ => Ok(())
            }
        }
    }

    pub fn is_class_loaded(class: &str) -> io::Result<()> {
        unsafe {
            let mut x: WNDCLASSEXW = mem::zeroed();
            match GetClassInfoExW(ptr::null_mut(), class.to_wide_null().as_ptr() as LPCWSTR, &mut x) {
                v if v == 0 => utils::last_error(),
                v => Ok(())
            }
        }
    }

    pub fn get_module_handle() -> io::Result<HMODULE> {
        unsafe {
            match GetModuleHandleW(ptr::null()) {
                v if v.is_null() => utils::last_error(),
                v => Ok(v)
            }
        }
    }
}
