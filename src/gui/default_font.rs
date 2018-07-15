use errors::MyErrorKind::WindowsError;
use failure::Error;
use failure::ResultExt;
use gui::Wnd;
use std::mem;
use winapi::shared::minwindef::*;
use winapi::shared::minwindef::TRUE;
use winapi::shared::windef::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;
use windows::utils::last_error;

unsafe extern "system" fn font_proc(wnd: HWND, font: LPARAM) -> BOOL {
    SendMessageW(wnd, WM_SETFONT, font as WPARAM, TRUE as LPARAM);
    TRUE
}

fn default_logfont() -> Result<LOGFONTW, Error> {
    unsafe {
        let mut metrics = mem::zeroed::<NONCLIENTMETRICSW>();
        let size = mem::size_of::<NONCLIENTMETRICSW>() as u32;
        metrics.cbSize = size;
        match SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            size,
            &mut metrics as *mut _ as *mut _,
            0)
            {
                v if v == 0 => last_error().context(WindowsError("SystemParametersInfoW failed"))?,
                _ => Ok(metrics.lfMessageFont),
            }
    }
}

fn default_font() -> Result<HFONT, Error> {
    unsafe {
        let font = default_logfont()?;
        match CreateFontIndirectW(&font) {
            v if v.is_null() => last_error().context(WindowsError("CreateFontIndirectW failed - default font"))?,
            v => Ok(v)
        }
    }
}

pub fn default_fonts() -> Result<(HFONT, HFONT), Error> {
    unsafe {
        let mut font = default_logfont()?;
        let default_font = match CreateFontIndirectW(&font) {
            v if v.is_null() => last_error().context(WindowsError("CreateFontIndirectW failed - default font"))?,
            v => v
        };
        font.lfWeight = FW_BOLD;
        let bold_font = match CreateFontIndirectW(&font) {
            v if v.is_null() => last_error().context(WindowsError("CreateFontIndirectW failed - bold font"))?,
            v => v
        };
        Ok((default_font, bold_font))
    }
}

pub fn set_font_on_children(parent: &Wnd) -> Result<(), Error> {
    unsafe {
        let font = default_font()?;
        EnumChildWindows(parent.hwnd, Some(font_proc), font as LPARAM);
        Ok(())
    }
}