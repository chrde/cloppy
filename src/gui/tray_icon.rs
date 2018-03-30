use winapi::um::shellapi::{
    NOTIFYICONDATAW,
    NOTIFYICONDATAW_u,
    NOTIFYICON_VERSION_4,
    Shell_NotifyIconW,
    NIF_MESSAGE,
    NIF_ICON,
    NIF_GUID,
    NIF_TIP,
    NIIF_NONE,
    NIM_ADD,
    NIM_DELETE,
};
use winapi::um::combaseapi::CoCreateGuid;
use winapi::shared::guiddef::GUID;
use winapi::um::winuser::{
    LoadImageW,
    DestroyIcon,
    CreateIconFromResourceEx,
    LookupIconIdFromDirectoryEx,
    LR_LOADFROMFILE,
    LR_DEFAULTCOLOR,
    LR_DEFAULTSIZE,
    IMAGE_ICON,
};
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::{
    HICON,
};
use winapi::shared::winerror::SUCCEEDED;

use gui::utils::ToWide;
use gui::wnd;

use std::mem;
use std::io;
use std::ptr;
use gui::utils;
use resources;

pub struct TrayIcon {
    data: NOTIFYICONDATAW
}

impl TrayIcon {
    unsafe fn load_icon_byte() -> io::Result<HICON> {
        match LookupIconIdFromDirectoryEx(resources::ICON.as_ptr() as *mut _, 1, 32, 32, LR_DEFAULTCOLOR) as isize {
            0 => utils::last_error(),
            offset => {
                match CreateIconFromResourceEx(resources::ICON.as_ptr().offset(offset) as *mut _, resources::ICON.len() as u32, 1, 0x00030000, 0, 0, LR_DEFAULTCOLOR) {
                    v if v.is_null() => utils::last_error(),
                    v => Ok(v)
                }
            }
        }
    }

    fn remove(&mut self) -> io::Result<()> {
        unsafe {
            match Shell_NotifyIconW(NIM_DELETE, &mut self.data) {
                v if v == 0 => utils::last_error(),
                _ => Ok(())
            }
        }
    }

    pub fn set_visible(&mut self) -> io::Result<()> {
        unsafe {
            match Shell_NotifyIconW(NIM_ADD, &mut self.data) {
                v if v == 0 => utils::last_error(),
                _ => Ok(())
            }
        }
    }

    pub fn new(wnd: &wnd::Wnd) -> Self {
        unsafe {
            let mut sz_tip: [u16; 128] = [0; 128];
            for (i, item) in "Cloppy".to_wide_null().iter().enumerate() {
                sz_tip[i] = *item;
            }
            let sz_info: [u16; 256] = [0; 256];
            let sz_info_title: [u16; 64] = [0; 64];
            let mut notify_version = mem::zeroed::<NOTIFYICONDATAW_u>();
            {
                let u_version = notify_version.uVersion_mut();
                *u_version = NOTIFYICON_VERSION_4;
            }
            let data = NOTIFYICONDATAW {
                cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: wnd.hwnd,
                uID: 0,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_GUID | NIF_TIP,
                uCallbackMessage: ::WM_SYSTRAYICON,
                hIcon: TrayIcon::load_icon_byte().unwrap(),
                szTip: sz_tip,
                dwState: 0,
                dwStateMask: 0,
                szInfo: sz_info,
                u: notify_version,
                szInfoTitle: sz_info_title,
                dwInfoFlags: NIIF_NONE,
                guidItem: TrayIcon::uuid().unwrap(),
                hBalloonIcon: ptr::null_mut(),
            };
            TrayIcon { data }
        }
    }

    unsafe fn uuid() -> io::Result<GUID> {
        let mut result: GUID = mem::zeroed();
        match SUCCEEDED(CoCreateGuid(&mut result)) {
            true => Ok(result),
            false => utils::other_error("Failed to create GUID")
        }
    }

    unsafe fn load_icon_from_file() -> io::Result<HICON> {
        match LoadImageW(
            ptr::null_mut(),
            "resources/cloppy_32.ico".to_wide_null().as_ptr() as LPCWSTR,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_LOADFROMFILE) {
            v if v.is_null() => utils::last_error(),
            v => Ok(v as HICON)
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            self.remove().unwrap();
            let result = match DestroyIcon(self.data.hIcon) {
                0 => utils::last_error(),
                _ => Ok(()),
            };
            result.unwrap();
            println!("icon destroyed");
        }
    }
}
