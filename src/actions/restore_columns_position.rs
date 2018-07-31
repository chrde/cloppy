use failure::Error;
use failure::ResultExt;
use gui::event::Event;
use gui::Gui;
use gui::Wnd;
use settings::Setting;
use settings::setting_to_int;
use std::io;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::WPARAM;
use winapi::um::commctrl::LVM_SETCOLUMNWIDTH;

pub fn restore_columns_position(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    let wnd = gui.item_list().wnd();
    let settings = gui.settings();
    restore_column(0, setting_to_int(Setting::ColumnFileNameWidth, settings), wnd)?;
    restore_column(1, setting_to_int(Setting::ColumnFilePathWidth, settings), wnd)?;
    restore_column(2, setting_to_int(Setting::ColumnFileSizeWidth, settings), wnd)?;
    Ok(())
}

fn restore_column(index: i32, width: i32, wnd: &Wnd) -> Result<(), Error> {
    match wnd.send_message(LVM_SETCOLUMNWIDTH, index as WPARAM, width as LPARAM) {
        0 => Err(io::Error::last_os_error()).with_context(|e| {
            format!("LVM_SETCOLUMNWIDTH failed - index {} width {}: {}", index, width, e)
        })?,
        _ => Ok(())
    }
}