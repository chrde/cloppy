use file_listing::list::icon::Icons;
use file_listing::list::item::DisplayItem;
use file_listing::list::item::Match;
use gui::default_font::default_fonts;
use gui::event::Event;
use plugin::CustomDrawResult;
use plugin::DrawResult;
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
use winapi::um::winnt::LPWSTR;
use winapi::um::winuser::DrawTextExW;
use winapi::um::winuser::DT_CALCRECT;
use winapi::um::winuser::DT_END_ELLIPSIS;
use winapi::um::winuser::FillRect;

pub struct ItemPaint {
    default_font: HFONT,
    bold_font: HFONT,
    icons: Icons,
}

unsafe impl Send for ItemPaint {}

impl ItemPaint {
    pub fn create() -> ItemPaint {
        let (default_font, bold_font) = default_fonts().unwrap();
        let icons = Icons::create();
        ItemPaint {
            default_font,
            bold_font,
            icons,
        }
    }

    pub fn draw_item(&self, event: Event, items: &HashMap<u32, DisplayItem>) -> DrawResult {
        let draw = event.as_display_info().item;
        let item = items.get(&(draw.iItem as u32)).unwrap();
        match draw.iSubItem {
            0 => DrawResult::IGNORE,
            1 => DrawResult::SIMPLE(item.path.as_ptr() as LPWSTR),
            2 => DrawResult::SIMPLE(item.size.as_ptr() as LPWSTR),
            _ => unreachable!()
        }
    }

    pub fn custom_draw_item(&self, event: Event, items: &HashMap<u32, DisplayItem>) -> CustomDrawResult {
        let custom_draw = event.as_custom_draw();
        if custom_draw.iSubItem == 0 {
            let item = items.get(&(custom_draw.nmcd.dwItemSpec as u32)).unwrap();
            self.draw_name(custom_draw, &item.matches);
            CustomDrawResult::HANDLED
        } else {
            CustomDrawResult::IGNORED
        }
    }

    pub fn draw_name(&self, draw_item: &NMLVCUSTOMDRAW, matches: &[Match]) {
        unsafe { FillRect(draw_item.nmcd.hdc, &draw_item.nmcd.rc as *const _, LTGRAY_BRUSH as HBRUSH); }
//        position.left += self.icons.draw_icon(&item, position, draw_item.nmcd.hdc);

        draw_text_with_matches(self.default_font, self.bold_font, &matches, draw_item.nmcd.hdc, draw_item.nmcd.rc);
    }

}

fn draw_text_with_matches(default_font: HFONT, bold_font: HFONT, matches: &[Match], hdc: HDC, pos: RECT) -> RECT {
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