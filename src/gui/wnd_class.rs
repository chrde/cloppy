use super::utils::ToWide;
use std::{io, mem, ptr};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM, HMODULE};
use winapi::um::winuser::{
    RegisterClassExW,
    UnregisterClassW,
    WNDCLASSEXW,
    GetWindowLongPtrW,
    GWL_HINSTANCE,
};
use gui::utils;

pub type WndProcRef = unsafe extern "system" fn(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;

pub struct WndClass(pub LPCWSTR, pub HINSTANCE);

impl WndClass {
    pub fn new(class_name: &str, wnd_proc: WndProcRef) -> io::Result<Self> {
        unsafe {
            let class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
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

    pub fn get_module_handle() -> io::Result<HMODULE> {
        unsafe {
            match GetModuleHandleW(ptr::null()) {
                v if v.is_null() => utils::last_error(),
                v => Ok(v)
            }
        }
    }
}

impl Drop for WndClass {
    fn drop(&mut self) {
        unsafe {
            let result = match UnregisterClassW(self.0, self.1) {
                0 => utils::last_error(),
                _ => Ok(())
            };
            result.unwrap()
        }
    }
}
