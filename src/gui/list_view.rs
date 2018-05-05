use gui::FILE_LIST_ID;
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
use winapi::um::shellapi::*;
use winapi::um::winnt::FILE_ATTRIBUTE_NORMAL;
use winapi::um::objbase::CoInitialize;
use gui::context_stash::CONTEXT_STASH;
use std::ptr;
use Message;
use gui::context_stash::send_message;
use gui::Wnd;
use file_listing::State;

pub fn new(parent: HWND, instance: Option<HINSTANCE>) -> io::Result<wnd::Wnd> {
    let list_view_params = wnd::WndParams::builder()
        .instance(instance)
        .window_name(get_string(FILE_LIST_NAME))
        .class_name(get_string(WC_LISTVIEW))
        .h_menu(FILE_LIST_ID as HMENU)
        .style(WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_OWNERDATA | LVS_ALIGNLEFT | LVS_SHAREIMAGELISTS | WS_CHILD)
        .h_parent(parent)
        .build();
    let list_view = wnd::Wnd::new(list_view_params)?;
    new_column(list_view.hwnd, 0, get_string("file_name"), "file_name".len() as i32);
    new_column(list_view.hwnd, 1, get_string("file_path"), "file_path".len() as i32);
    new_column(list_view.hwnd, 2, get_string("file_size"), "file_size".len() as i32);
    unsafe {
        CoInitialize(ptr::null_mut());
        let mut info = mem::zeroed::<SHFILEINFOW>();
        let image_list = SHGetFileInfoW(
            get_string("C:\\"),
            FILE_ATTRIBUTE_NORMAL,
            &mut info as *mut _,
            mem::size_of::<SHFILEINFOW> as u32,
            SHGFI_SYSICONINDEX | SHGFI_SMALLICON | SHGFI_ICON | SHGFI_USEFILEATTRIBUTES);
        assert_ne!(image_list, 0);
        SendMessageW(list_view.hwnd, LVM_SETIMAGELIST, LVSIL_SMALL as WPARAM, image_list as LPARAM);
    }
    unsafe { SendMessageW(list_view.hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as WPARAM, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as LPARAM); };
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
    unsafe { SendMessageW(wnd, LVM_INSERTCOLUMNW, index as WPARAM, &column as *const _ as LPARAM); };
    column
}

pub unsafe fn on_cache_hint(event: Event) {
    let hint = *(event.l_param as LPNMLVCACHEHINT);
    let message = Message::LOAD(hint.iFrom as u32..hint.iTo as u32 + 1);
    send_message(message);
}

pub unsafe fn on_get_display_info(event: Event) {
    use gui::utils::ToWide;
    let plvdi = *(event.l_param as LPNMLVDISPINFOW);
    if (plvdi.item.mask & LVIF_IMAGE) == LVIF_IMAGE {
        (*(event.l_param as LPNMLVDISPINFOW)).item.iImage = plvdi.item.iItem;
    }
    if (plvdi.item.mask & LVIF_TEXT) == LVIF_TEXT {
        CONTEXT_STASH.with(|context_stash| {
            let list_item = &mut (*(event.l_param as LPNMLVDISPINFOW)).item;
            let mut context_stash = context_stash.borrow_mut();
            let local_data = context_stash.as_mut().unwrap();
            match plvdi.item.iSubItem {
                0 => {
                    let value = local_data.arena.name_of(list_item.iItem as usize + local_data.state.items_start());
                    list_item.pszText = value.to_wide_null().as_ptr() as LPWSTR;//ptr::null_mut();
                }
                1 => {
                    list_item.pszText = (list_item.iItem.to_string() + "asdf").to_wide_null().as_ptr() as LPWSTR;
                }
                2 => {
                    list_item.pszText = (list_item.iItem.to_string() + "qwert").to_wide_null().as_ptr() as LPWSTR;
                }
                _ => {
                    println!("WTF");
                    unreachable!();
                }
            }
        });
    }
}

pub struct ItemList {
    wnd: Wnd,
    header: ListHeader,
}

impl ItemList {
    pub fn new(wnd: Wnd, header: Wnd) -> ItemList {
        let header = ListHeader { wnd: header };
        ItemList { wnd, header }
    }

    pub fn wnd(&self) -> &Wnd {
        &self.wnd
    }

    pub unsafe fn on_header_click(&self, event: Event) {
        self.header.add_sort_arrow_to_header(event);
    }

    pub fn update(&self, state: &State) {
        let size = state.count();
        self.wnd.send_message(LVM_SETITEMCOUNT, size as WPARAM, 0);
    }
}

struct ListHeader {
    wnd: Wnd,
}

impl ListHeader {
    unsafe fn add_sort_arrow_to_header(&self, event: Event) {
        let pnmv = *(event.l_param as LPNMLISTVIEW);
        assert!(pnmv.iSubItem >= 0);
        let mut item = mem::zeroed::<HDITEMW>();
        item.mask = HDI_FORMAT;
        self.wnd.send_message(HDM_GETITEMW, pnmv.iSubItem as WPARAM, &mut item as *mut _ as LPARAM);
        item.fmt = next_order(item.fmt);
        self.wnd.send_message(HDM_SETITEMW, pnmv.iSubItem as WPARAM, &mut item as *mut _ as LPARAM);
    }
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
