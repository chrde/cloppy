use file_listing::file_type_icon::IconRetriever;
use file_listing::list::item::*;
use file_listing::State;
use gui::context_stash::send_message;
use gui::default_font::default_fonts;
use gui::event::Event;
use gui::FILE_LIST_ID;
use gui::FILE_LIST_NAME;
use gui::get_string;
use gui::list_header::ListHeader;
use gui::wnd;
use gui::Wnd;
use Message;
use sql::arena::Arena;
use std::cmp;
use std::collections::HashMap;
use std::io;
use std::mem;
use std::ptr;
use std::sync::Arc;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::commctrl::WC_LISTVIEW;
use winapi::um::wingdi::*;
use winapi::um::wingdi::SelectObject;
use winapi::um::winuser::*;
use winapi::um::winuser::DRAWITEMSTRUCT;


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
    items_cache: HashMap<i32, DisplayItem>,
    default_font: HFONT,
    bold_font: HFONT,
}

pub struct DisplayItem1 {
    pub name: String,
    pub path: Vec<u16>,
    pub size: Vec<u16>,
    pub matches: Vec<Match>,
    pub flags: u8,
}

impl DisplayItem1 {
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


    fn draw_item_icon(&self, item: &DisplayItem, mut position: RECT, hdc: HDC) -> RECT {
        let icon = self.icon_cache.get(item);
        unsafe {
            ImageList_Draw(icon.image_list as HIMAGELIST, icon.index, hdc, position.left, position.top, ILD_TRANSPARENT);
        }
        position.left += icon.width;
        position
    }

    fn painting_position_of(&self, draw_item: &DRAWITEMSTRUCT, header_pos: usize) -> RECT {
        let mut position = draw_item.rcItem;
        let offset = self.header.offset_of(header_pos);
        position.left += offset;
        position.right += offset;
        let max_width = position.left + self.header.width_of(header_pos);
        position.right = cmp::min(max_width, position.right);
        position
    }

    fn draw_item_name(&self, draw_item: &DRAWITEMSTRUCT, header_pos: usize) {
        let mut position = self.painting_position_of(draw_item, header_pos);

        let item = self.items_cache.get(&(draw_item.itemID as i32)).unwrap();
        unsafe { FillRect(draw_item.hDC, &position as *const _, LTGRAY_BRUSH as HBRUSH); }
        position = self.draw_item_icon(&item, position, draw_item.hDC);

        draw_text_with_matches(self.default_font, self.bold_font, &item.matches, draw_item.hDC, &mut position, &item.name);
    }

    fn draw_item_part(&self, draw_item: &DRAWITEMSTRUCT, header_pos: usize, text: &[u16]) {
        let mut position = self.painting_position_of(draw_item, header_pos);
        draw_text_section(self.default_font, draw_item.hDC, &mut position, text);
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
            let position = state.items()[item.iItem as usize].clone();
            let display_item = arena.file(position, &state.query());
            self.items_cache.insert(item.iItem, display_item);
        }
    }
}

fn draw_text_with_matches(default_font: HFONT, bold_font: HFONT, matches: &[Match], hdc: HDC, pos: &mut RECT, text: &String) -> RECT {
    let mut position = pos.clone();
    for m in matches {
        let font = if m.matched { bold_font } else { default_font };
        let mut rect = draw_text_section(font, hdc, &mut position, &text[m.init..m.end].encode_utf16().collect::<Vec<_>>());
        position.left += rect.right;
    };
    position
}

fn draw_text_section(font: HFONT, hdc: HDC, pos: &mut RECT, text: &[u16]) -> RECT {
    let mut next_position = unsafe { mem::zeroed::<RECT>() };
    unsafe { SelectObject(hdc, font as HGDIOBJ); }
    unsafe { DrawTextExW(hdc, text.as_ptr(), text.len() as i32, &mut next_position as *mut _, DT_CALCRECT, ptr::null_mut()) };
    unsafe { DrawTextExW(hdc, text.as_ptr(), text.len() as i32, pos as *mut _, DT_END_ELLIPSIS, ptr::null_mut()) };
    next_position
}