use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::TRUE;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;
use gui::utils;
use std::io;
use std::mem;
use gui::wnd_proc::Event;

unsafe extern "system" fn font_proc(wnd: HWND, font: LPARAM) -> BOOL {
    SendMessageW(wnd, WM_SETFONT, font as WPARAM, TRUE as LPARAM);
    TRUE
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

pub unsafe fn set_font_on_children(event: Event) {
    EnumChildWindows(event.wnd, Some(font_proc), default_font().unwrap() as LPARAM);
}