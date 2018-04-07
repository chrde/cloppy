use gui::utils;
use gui::utils::ToWide;
use std::{io, ptr};
use winapi::um::winuser::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::{
    HWND,
    HMENU,
};
use winapi::shared::ntdef::LPCWSTR;

pub struct Wnd {
    pub hwnd: HWND,
}

impl Wnd {
    pub fn new(params: WndParams) -> io::Result<Self> {
        unsafe {
            match CreateWindowExW(
                params.ex_style,
                params.class_name,
                params.window_name.to_wide_null().as_ptr(),
                params.style,
                params.location.x,
                params.location.y,
                params.width,
                params.height,
                params.h_parent,
                params.h_menu,
                params.instance,
                params.lp_param,
            ) {
                v if v.is_null() => utils::last_error(),
                v => {
                    println!("Created window {:?}{:?}", params.window_name, v);
                    Ok(Wnd { hwnd: v })
                }
            }
        }
    }

    pub fn show(&self, mode: INT) -> BOOL {
        unsafe {
            ShowWindow(self.hwnd, mode)
        }
    }

    pub fn update(&self) -> io::Result<()> {
        unsafe {
            match UpdateWindow(self.hwnd) {
                0 => utils::last_error(),
                _ => Ok(())
            }
        }
    }
}

#[derive(TypedBuilder)]
pub struct WndParams<'a> {
    window_name: &'a str,
    class_name: LPCWSTR,
    instance: HINSTANCE,
    style: DWORD,
    #[default = "0"]
    ex_style: DWORD,
    #[default = "ptr::null_mut()"]
    h_parent: HWND,
    #[default = "ptr::null_mut()"]
    h_menu: HMENU,
    #[default = "ptr::null_mut()"]
    lp_param: LPVOID,
    #[default = "CW_USEDEFAULT"]
    width: INT,
    #[default = "CW_USEDEFAULT"]
    height: INT,
    #[default = "Default::default()"]
    location: utils::Location,
}

