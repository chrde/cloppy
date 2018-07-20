use failure::Error;
use failure::ResultExt;
use gui::event::Event;
use gui::Wnd;
use num::FromPrimitive;
use slog::Logger;
use std::io;
use winapi::shared::minwindef::WPARAM;
use winapi::um::winuser::MOD_ALT;
use winapi::um::winuser::MOD_NOREPEAT;
use winapi::um::winuser::MOD_WIN;
use winapi::um::winuser::RegisterHotKey;
use winapi::um::winuser::SetForegroundWindow;
use winapi::um::winuser::SW_SHOW;

const N_KEY: u32 = 0x4E;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum Shortcut {
    ShowFilesWindow,
}
}

impl Shortcut {
    pub fn from_wparam(w_param: WPARAM) -> Option<Shortcut> {
        Shortcut::from_i32(w_param as i32)
    }
}

pub fn on_hotkey_event(logger: &Logger, event: Event) {
    let id = event.w_param() as i32;
    match Shortcut::from_i32(id) {
        None => warn!(logger, "unknown shortcut"; "id" => id, "type" => "shortcut"),
        Some(shortcut) => {
            info!(logger, "handling shortcut"; "id" => ?shortcut, "type" => "shortcut");
            handle_shortcut(shortcut, event);
        }
    }
}

fn handle_shortcut(shortcut: Shortcut, event: Event) {
    match shortcut {
        Shortcut::ShowFilesWindow => {
            event.wnd().show(SW_SHOW);
            event.wnd().set_as_foreground();
        }
    };
}

pub fn register_global_files(wnd: &Wnd) -> Result<(), Error> {
    unsafe {
        match RegisterHotKey(wnd.hwnd, Shortcut::ShowFilesWindow as i32, (MOD_WIN | MOD_ALT | MOD_NOREPEAT) as u32, N_KEY) {
            v if v == 0 => Err(io::Error::last_os_error()).with_context(|e| {
                let key = "WIN + ALT + N";
                format!("Could not register key {}: {}", key, e)
            })?,
            _ => Ok(()),
        }
    }
}