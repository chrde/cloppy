use gui::context_stash::apply_on_window;
use gui::FILE_LIST_HEADER_ID;
use gui::FILE_LIST_ID;
use gui::HASHMAP;
use gui::wnd;
use gui::wnd_proc::Event;
use std::io;
use std::mem;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use gui::get_string;
use gui::FILE_LIST_NAME;
use winapi::um::commctrl::WC_LISTVIEW;

pub fn new(parent: HWND, instance: Option<HINSTANCE>) -> io::Result<wnd::Wnd> {
    let list_view_params = wnd::WndParams::builder()
        .instance(instance)
        .window_name(get_string(FILE_LIST_NAME))
        .class_name(get_string(WC_LISTVIEW))
        .h_menu(FILE_LIST_ID as HMENU)
        .style(WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_OWNERDATA | LVS_ALIGNLEFT | WS_CHILD)
        .h_parent(parent)
        .build();
    let list_view = wnd::Wnd::new(list_view_params)?;
    new_column(list_view.hwnd, 0, get_string("column"), "column".len() as i32);
    new_column(list_view.hwnd, 1, get_string("column"), "column".len() as i32);
    new_column(list_view.hwnd, 2, get_string("column"), "column".len() as i32);
    unsafe { SendMessageW(list_view.hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as WPARAM, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as LPARAM); };
    unsafe { SendMessageW(list_view.hwnd, LVM_SETITEMCOUNT, 20, 0); };
    Ok(list_view)
}

fn new_column(wnd: HWND, index: i32, text: LPCWSTR, len: i32) -> LVCOLUMNW {
    let mut column = unsafe { mem::zeroed::<LVCOLUMNW>() };
    column.cx = 200;
    column.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_ORDER;
    column.pszText = text as LPWSTR;
    column.cchTextMax = len as i32;
    column.iSubItem = index;
    column.iOrder = index;
    unsafe { SendMessageW(wnd, LVM_INSERTCOLUMNW, 0, &column as *const _ as LPARAM); };
    column
}

pub unsafe fn on_get_display_info(event: Event) {
    let plvdi = *(event.l_param as LPNMLVDISPINFOW);
    if (plvdi.item.mask & LVIF_TEXT) == LVIF_TEXT {
        (*(event.l_param as LPNMLVDISPINFOW)).item.pszText = HASHMAP.lock().get("hello").unwrap().as_ptr() as LPWSTR;
//                        match plvdi.item.iSubItem {
//                            0 => {
//                                (*(l_param as LPNMLVDISPINFOW)).item.pszText = HASHMAP.get(&0).unwrap().as_ptr() as LPWSTR;
//                            }
//                            2 => {
//                                println!("asking for {} {}", plvdi.item.iItem, plvdi.item.iSubItem);
//                                plvdi.item.pszText = "column 2".to_wide_null().as_ptr() as LPWSTR;
//                            }
//                            _ => {
//                                println!("WTF");
//                                unreachable!();
//                            }
//                        }
    }
}

pub unsafe fn on_header_click(event: Event) {
    add_sort_arrow_to_header(event);
}

unsafe fn add_sort_arrow_to_header(event: Event) {
    let pnmv = *(event.l_param as LPNMLISTVIEW);
    assert!(pnmv.iSubItem >= 0);
    apply_on_window(FILE_LIST_HEADER_ID, |ref wnd| {
        let mut item = mem::zeroed::<HDITEMW>();
        item.mask = HDI_FORMAT;
        SendMessageW(wnd.hwnd, HDM_GETITEMW, pnmv.iSubItem as WPARAM, &mut item as *mut _ as LPARAM);
        item.fmt = next_order(item.fmt);
        SendMessageW(wnd.hwnd, HDM_SETITEMW, pnmv.iSubItem as WPARAM, &mut item as *mut _ as LPARAM);
    });
}

fn next_order(current: i32) -> i32 {
    match current {
        v if (v & HDF_SORTDOWN) == HDF_SORTDOWN => v & !HDF_SORTDOWN,
        v if (v & HDF_SORTUP) == HDF_SORTUP => (v & !HDF_SORTUP) | HDF_SORTDOWN,
        v => v | HDF_SORTUP,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_order_to_ascending() {
        assert_eq!(HDF_SORTUP, next_order(0));
    }

    #[test]
    fn ascending_order_to_descending() {
        assert_eq!(HDF_SORTDOWN, next_order(HDF_SORTUP));
    }

    #[test]
    fn descending_order_to_none() {
        assert_eq!(0, next_order(HDF_SORTDOWN));
    }

    #[test]
    fn next_order_keeps_other_fmt() {
        assert_eq!(HDF_SORTUP + 1, next_order(1));
        assert_eq!(HDF_SORTDOWN + 1, next_order(HDF_SORTUP + 1));
        assert_eq!(1, next_order(HDF_SORTDOWN + 1));
    }
}