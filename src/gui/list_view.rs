use gui::FILE_LIST_ID;
use gui::wnd;
use std::io;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use gui::get_string;
use gui::FILE_LIST_NAME;
use winapi::um::commctrl::WC_LISTVIEW;
use Message;
use gui::context_stash::send_message;
use gui::Wnd;
use file_listing::State;
use std::sync::Arc;
use sql::Arena;
use gui::event::Event;
use file_listing::file_type_icon::IconRetriever;
use std::collections::HashMap;
use winapi::um::winuser::DRAWITEMSTRUCT;
use gui::utils::ToWide;
use winapi::um::wingdi::SelectObject;
use file_listing::Match;
use gui::default_font::default_fonts;
use winapi::um::wingdi::*;
use std::cmp;
use gui::list_header::ListHeader;


pub fn create(parent: HWND, instance: Option<HINSTANCE>) -> ItemList {
    let (default, bold) = default_fonts().unwrap();
    let (list, header) = new(parent, instance).unwrap();
    ItemList::new(list, header, default, bold)
}

fn new(parent: HWND, instance: Option<HINSTANCE>) -> io::Result<(wnd::Wnd, ListHeader)> {
    let list_view_params = wnd::WndParams::builder()
        .instance(instance)
        .window_name(get_string(FILE_LIST_NAME))
        .class_name(get_string(WC_LISTVIEW))
        .h_menu(FILE_LIST_ID as HMENU)
        .style(WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_OWNERDATA | LVS_ALIGNLEFT | LVS_SHAREIMAGELISTS | LVS_OWNERDRAWFIXED | WS_CHILD)
        .h_parent(parent)
        .build();
    let list_view = wnd::Wnd::new(list_view_params)?;
    unsafe { SendMessageW(list_view.hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as WPARAM, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as LPARAM); };
    let header = ListHeader::create(&list_view);
    Ok((list_view, header))
}

pub fn on_cache_hint(event: Event) {
    let hint = event.as_cache_hint();
    let message = Message::LOAD(hint.iFrom as u32..hint.iTo as u32 + 1);
    send_message(message);
}

pub struct ItemList {
    wnd: Wnd,
    header: ListHeader,
    icon_cache: IconRetriever,
    items_cache: HashMap<i32, Item>,
    default_font: HFONT,
    bold_font: HFONT,
}

pub struct Item {
    pub name: String,
    path: Vec<u16>,
    size: Vec<u16>,
    matches: Vec<Match>,
    pub flags: u8,
}

impl Item {
    pub fn is_directory(&self) -> bool {
        self.flags & 2 != 0
    }
}

impl ItemList {
    fn new(wnd: Wnd, header: ListHeader, default_font: HFONT, bold_font: HFONT) -> ItemList {
        let items_cache = HashMap::new();
        let icon_cache = IconRetriever::create();
        ItemList { wnd, header, icon_cache, items_cache, default_font, bold_font }
    }

    pub fn scroll_to_top(&self) {
        self.wnd.send_message(LVM_ENSUREVISIBLE, 0, false as isize);
    }

    pub fn wnd(&self) -> &Wnd {
        &self.wnd
    }

    pub fn on_header_click(&mut self, event: Event) {
        self.header.add_sort_arrow_to_header(event);
    }

    pub fn on_header_change(&mut self, event: Event) {
        self.header.update_widths(event);
        self.wnd.invalidate_rect(None, true);
    }

    pub fn update(&self, state: &State) {
        self.scroll_to_top();
        self.wnd.send_message(LVM_SETITEMCOUNT, state.count() as WPARAM, 0);
    }

    fn draw_text_section(&self, font: HFONT, hdc: HDC, pos: &mut RECT, text: &[u16], max_width: i32) -> RECT {
        let mut next_position = unsafe { mem::zeroed::<RECT>() };
        let right = cmp::min(max_width, pos.right);
        pos.right = right;
        unsafe { SelectObject(hdc, font as HGDIOBJ); }
        unsafe { DrawTextExW(hdc, text.as_ptr(), text.len() as i32, &mut next_position as *mut _, DT_CALCRECT, ptr::null_mut()) };
        unsafe { DrawTextExW(hdc, text.as_ptr(), text.len() as i32, pos as *mut _, DT_END_ELLIPSIS, ptr::null_mut()) };
        next_position
    }

    fn draw_item_icon(&self, item: &Item, mut position: RECT, hdc: HDC) -> RECT {
        let icon = self.icon_cache.get(item);
        unsafe {
            ImageList_Draw(icon.image_list as HIMAGELIST, icon.index, hdc, position.left, position.top, ILD_TRANSPARENT);
        }
        position.left += icon.width;
        position
    }

    fn draw_item_name(&self, draw_item: &DRAWITEMSTRUCT, header_pos: usize) {
        let mut position = draw_item.rcItem;
        let max_width = position.left + self.header.width_of(header_pos);
        let item = self.items_cache.get(&(draw_item.itemID as i32)).unwrap();
        position.right = max_width;
        unsafe { FillRect(draw_item.hDC, &position as *const _, LTGRAY_BRUSH as HBRUSH); }
        position = self.draw_item_icon(&item, position, draw_item.hDC);
        for m in &item.matches {
            let font = if m.matched { self.bold_font } else { self.default_font };
            let mut rect = self.draw_text_section(font, draw_item.hDC, &mut position, &item.name[m.init..m.end].encode_utf16().collect::<Vec<_>>(), max_width);
            position.left += rect.right;
        }
    }

    fn draw_item_part(&self, draw_item: &DRAWITEMSTRUCT, header_pos: usize, text: &[u16]) {
        let mut position = draw_item.rcItem;
        let offset = self.header.offset_of(header_pos);
        position.left += offset;
        position.right += offset;
        let max_width = position.left + self.header.width_of(header_pos);
        self.draw_text_section(self.default_font, draw_item.hDC, &mut position, text, max_width);
    }

    pub fn draw_item(&mut self, event: Event, _state: &State) {
        let draw_item = event.as_draw_item();

        match draw_item.itemAction {
            ODA_DRAWENTIRE => {
                let item = self.items_cache.get(&(draw_item.itemID as i32)).unwrap();
                self.draw_item_name(draw_item, 0);
                self.draw_item_part(draw_item, 1, &item.path);
                self.draw_item_part(draw_item, 2, &item.size);
            }
            /*
            if (Item->itemState & ODS_FOCUS)
                {
                    DrawFocusRect(Item->hDC, &Item->rcItem);
                }
                */
            _ => panic!("other"),
        }
    }

    pub fn display_item(&mut self, event: Event, arena: &Arc<Arena>, state: &State) {
        let item = &mut event.as_display_info().item;
        if (item.mask & LVIF_IMAGE) == LVIF_IMAGE {
//            let position = state.items()[item.iItem as usize];
//            let name = arena.name_of(position);
//            item.iImage = self.icon_cache.get(name);
        }
        if (item.mask & LVIF_TEXT) == LVIF_TEXT {
            let position = state.items()[item.iItem as usize];
            let file = arena.file(position).unwrap();
            let name = arena.name_of(position);
            let matches = state.matches(name);
            let cached_item = Item {
                name: name.to_owned(),
                path: arena.path_of(position).to_wide_null(),
                size: file.size().to_string().to_wide_null(),
                matches,
                flags: file.flags(),
            };
            self.items_cache.insert(item.iItem, cached_item);
        }
    }
}
