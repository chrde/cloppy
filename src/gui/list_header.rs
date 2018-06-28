use gui::event::Event;
use gui::get_string;
use gui::wnd::Wnd;
use std::mem;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;

const COLUMN_WIDTH: i32 = 200;

pub struct ListHeader {
    wnd: Wnd,
    sorted_by_column: usize,
}

impl ListHeader {
    pub fn create(list: &Wnd) -> ListHeader {
        let column = new_column(0, get_string("file_name"), "file_name".len() as i32);
        list.send_message(LVM_INSERTCOLUMNW, column.iSubItem as WPARAM, &column as *const _ as LPARAM);
        let column = new_column(1, get_string("file_path"), "file_path".len() as i32);
        list.send_message(LVM_INSERTCOLUMNW, column.iSubItem as WPARAM, &column as *const _ as LPARAM);
        let column = new_column_right_aligned(2, get_string("file_size"), "file_size".len() as i32);
        list.send_message(LVM_INSERTCOLUMNW, column.iSubItem as WPARAM, &column as *const _ as LPARAM);
        let hwnd = list.send_message(LVM_GETHEADER, 0, 0) as HWND;
        ListHeader {
            wnd: Wnd { hwnd },
            sorted_by_column: 0,
        }
    }

    fn reset_old_header(&self) {
        let mut item = unsafe { mem::zeroed::<HDITEMW>() };
        item.mask = HDI_FORMAT;
        self.wnd.send_message(HDM_GETITEMW, self.sorted_by_column as WPARAM, &mut item as *mut _ as LPARAM);
        item.fmt = reset_order(item.fmt);
        self.wnd.send_message(HDM_SETITEMW, self.sorted_by_column as WPARAM, &mut item as *mut _ as LPARAM);
    }

    pub fn add_sort_arrow_to_header(&mut self, event: Event) {
        let list_view = event.as_list_view();
        assert!(list_view.iSubItem >= 0);
        if list_view.iSubItem as usize != self.sorted_by_column {
            self.reset_old_header();
            self.sorted_by_column = list_view.iSubItem as usize;
        }
        let mut item = unsafe { mem::zeroed::<HDITEMW>() };
        item.mask = HDI_FORMAT;
        self.wnd.send_message(HDM_GETITEMW, self.sorted_by_column as WPARAM, &mut item as *mut _ as LPARAM);
        item.fmt = next_order(item.fmt);
        self.wnd.send_message(HDM_SETITEMW, self.sorted_by_column as WPARAM, &mut item as *mut _ as LPARAM);
    }
}

fn next_order(current: i32) -> i32 {
    match current {
        v if (v & HDF_SORTDOWN) == HDF_SORTDOWN => v & !HDF_SORTDOWN,
        v if (v & HDF_SORTUP) == HDF_SORTUP => (v & !HDF_SORTUP) | HDF_SORTDOWN,
        v => v | HDF_SORTUP,
    }
}

fn reset_order(current: i32) -> i32 {
    current & !HDF_SORTUP & !HDF_SORTDOWN
}


fn new_column(index: i32, text: LPCWSTR, len: i32) -> LVCOLUMNW {
    let mut column = unsafe { mem::zeroed::<LVCOLUMNW>() };
    column.cx = COLUMN_WIDTH;
    column.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_ORDER | LVCF_FMT;
    column.fmt = LVCFMT_LEFT;
    column.pszText = text as LPWSTR;
    column.cchTextMax = len as i32;
    column.iSubItem = index;
    column.iOrder = index;
    column
}

fn new_column_right_aligned(index: i32, text: LPCWSTR, len: i32) -> LVCOLUMNW {
    let mut column = new_column(index, text, len);
    column.fmt = LVCFMT_RIGHT;
    column
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_order() {
        let value = 42;
        assert_eq!(value, reset_order(value));
        assert_eq!(value, reset_order(next_order(value)));
        assert_eq!(value, reset_order(next_order(next_order(value))));
        assert_eq!(value, reset_order(next_order(next_order(next_order(value)))));
    }

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