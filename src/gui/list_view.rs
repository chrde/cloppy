use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use gui::wnd;
use FILE_LIST_ID;
use std::io;
use gui::utils::Location;
use gui::utils::ToWide;
use std::mem;


pub fn list_view(parent: HWND) -> io::Result<wnd::Wnd> {
    let list_view_params = wnd::WndParams::builder()
        .window_name("mylistview")
        .class_name(WC_LISTVIEW.to_wide_null().as_ptr() as LPCWSTR)
        .h_menu(FILE_LIST_ID as HMENU)
        .style(WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_OWNERDATA | LVS_ALIGNLEFT | WS_CHILD)
        .h_parent(parent)
        .location(Location { x: 0, y: 30 })
        .height(300)
        .width(300)
        .build();
    let list_view = wnd::Wnd::new(list_view_params)?;
    new_column(list_view.hwnd, 0, "zero");
    new_column(list_view.hwnd, 1, "one");
    new_column(list_view.hwnd, 2, "two");
    unsafe { SendMessageW(list_view.hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as WPARAM, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as LPARAM); };
    unsafe { SendMessageW(list_view.hwnd, LVM_SETITEMCOUNT, 20000000, 0); };
    Ok(list_view)
}

fn new_column(wnd: HWND, index: i32, text: &str) -> LVCOLUMNW {
    let mut column = unsafe { mem::zeroed::<LVCOLUMNW>() };
    column.cx = 200;
    column.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_ORDER;
    column.pszText = text.to_wide_null().as_mut_ptr();
    column.cchTextMax = text.len() as i32;
    column.iSubItem = index;
    column.iOrder = index;
    unsafe { SendMessageW(wnd, LVM_INSERTCOLUMNW, 0, &column as *const _ as LPARAM); };
    column
}
