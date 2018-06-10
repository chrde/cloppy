use file_listing::files::Files;
use file_listing::list::icon::Icons;
use file_listing::list::item::DisplayItem;
use file_listing::list::item::Match;
use gui::default_font::default_fonts;
use gui::event::Event;
use plugin::ItemDraw;
use plugin::State;
use plugin::SuperMatch;
use std::collections::HashMap;
use std::mem;
use std::ptr;
use winapi::shared::windef::HBRUSH;
use winapi::shared::windef::HDC;
use winapi::shared::windef::HFONT;
use winapi::shared::windef::HGDIOBJ;
use winapi::shared::windef::RECT;
use winapi::um::commctrl::NMLVCUSTOMDRAW;
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

unsafe impl Send for ItemPaint {}

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

    pub fn get_item(&self, id: u32) -> &DisplayItem {
        self.items_cache.get(&id).unwrap()
    }

    pub fn draw_name(&self, draw_item: &NMLVCUSTOMDRAW, matches: &[SuperMatch]) {
        unsafe { FillRect(draw_item.nmcd.hdc, &draw_item.nmcd.rc as *const _, LTGRAY_BRUSH as HBRUSH); }
//        position.left += self.icons.draw_icon(&item, position, draw_item.nmcd.hdc);

        draw_text_with_matches(self.default_font, self.bold_font, &matches, draw_item.nmcd.hdc, draw_item.nmcd.rc);
    }

    pub fn prepare_item(&mut self, id: usize, files: &Files, state: &State) {
        let position = state.items()[id].clone();
        let file = files.file(position);
        let path = files.path_of(file);
        self.items_cache.insert(id as u32, DisplayItem::new(file, path, &state.query()));
    }
}

fn draw_text_with_matches(default_font: HFONT, bold_font: HFONT, matches: &[SuperMatch], hdc: HDC, pos: RECT) -> RECT {
    let mut position = pos.clone();
    for m in matches {
        let font = if m.matched { bold_font } else { default_font };
        let mut rect = draw_text_section(font, hdc, &mut position, &m.text);
        position.left += rect.right;
    };
    position
}

fn draw_text_section(font: HFONT, hdc: HDC, pos: &mut RECT, text: &[u16]) -> RECT {
    let mut next_position = unsafe { mem::zeroed::<RECT>() };
    let old = unsafe { SelectObject(hdc, font as HGDIOBJ) };
    unsafe { DrawTextExW(hdc, text.as_ptr(), text.len() as i32, &mut next_position as *mut _, DT_CALCRECT, ptr::null_mut()) };
    unsafe { DrawTextExW(hdc, text.as_ptr(), text.len() as i32, pos as *mut _, DT_END_ELLIPSIS, ptr::null_mut()) };
    unsafe { SelectObject(hdc, old as HGDIOBJ); }
    next_position
}