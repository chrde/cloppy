use file_listing::file_type_icon::Icons;
use file_listing::files::Files;
use file_listing::list::item::DisplayItem;
use file_listing::list::item::Match;
use file_listing::State;
use gui::default_font::default_fonts;
use gui::event::Event;
use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::sync::Arc;
use winapi::shared::windef::HBRUSH;
use winapi::shared::windef::HDC;
use winapi::shared::windef::HFONT;
use winapi::shared::windef::HGDIOBJ;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::LTGRAY_BRUSH;
use winapi::um::wingdi::SelectObject;
use winapi::um::winuser::DRAWITEMSTRUCT;
use winapi::um::winuser::DrawTextExW;
use winapi::um::winuser::DT_CALCRECT;
use winapi::um::winuser::DT_END_ELLIPSIS;
use winapi::um::winuser::FillRect;

pub struct ItemPaint {
    default_font: HFONT,
    bold_font: HFONT,
    items_cache: HashMap<u32, DisplayItem>,
    icons: Icons,
}

impl ItemPaint {
    pub fn create() -> ItemPaint {
        let (default_font, bold_font) = default_fonts().unwrap();
        let items_cache = HashMap::new();
        let icons = Icons::create();
        ItemPaint {
            default_font,
            bold_font,
            items_cache,
            icons,
        }
    }

    fn draw_column(&self, draw_item: &DRAWITEMSTRUCT, mut position: RECT, text: &[u16]) {
        draw_text_section(self.default_font, draw_item.hDC, &mut position, text);
    }

    pub fn draw_item(&mut self, event: Event, positions: [RECT; 3]) {
        let draw_item = event.as_draw_item();

        let item = self.items_cache.get(&draw_item.itemID).unwrap();
        self.draw_name(draw_item, positions[0]);
        self.draw_column(draw_item, positions[1], &item.path);
        self.draw_column(draw_item, positions[2], &item.size);
    }

    fn draw_name(&self, draw_item: &DRAWITEMSTRUCT, mut position: RECT) {
        let item = self.items_cache.get(&draw_item.itemID).unwrap();
        unsafe { FillRect(draw_item.hDC, &position as *const _, LTGRAY_BRUSH as HBRUSH); }
        position.left += self.icons.draw_icon(&item, position, draw_item.hDC);

        draw_text_with_matches(self.default_font, self.bold_font, &item.matches, draw_item.hDC, &mut position, &item.name);
    }

    pub fn prepare_item(&mut self, id: u32, arena: &Arc<Files>, state: &State) {
        let position = state.items()[id as usize].clone();
        let file = arena.file(position);
        let path = arena.path_of(file);
        self.items_cache.insert(id, DisplayItem::new(file, path, &state.query()));
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