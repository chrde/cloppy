use file_listing::list::paint::ItemPaint;
use gui::event::Event;
use gui::FILE_LIST_ID;
use gui::FILE_LIST_NAME;
use gui::get_string;
use gui::get_string_mut;
use gui::list_header::ListHeader;
use gui::wnd;
use gui::Wnd;
use plugin::ItemDraw;
use plugin::Plugin;
use plugin::State;
use plugin::SuperMatch;
use std::cmp;
use std::io;
use std::mem;
use std::ptr;
use std::sync::Arc;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::commctrl::WC_LISTVIEW;
use winapi::um::wingdi::SelectObject;
use winapi::um::winnt::LPWSTR;
use winapi::um::winuser::*;
use winapi::um::winuser::DRAWITEMSTRUCT;


pub fn create(parent: HWND, instance: Option<HINSTANCE>, plugin: Arc<Plugin>) -> ItemList {
    let (list, header) = new(parent, instance).unwrap();
    ItemList::new(list, header, plugin)
}

fn new(parent: HWND, instance: Option<HINSTANCE>) -> io::Result<(wnd::Wnd, ListHeader)> {
    let list_view_params = wnd::WndParams::builder()
        .instance(instance)
        .window_name(get_string(FILE_LIST_NAME))
        .class_name(get_string(WC_LISTVIEW))
        .h_menu(FILE_LIST_ID as HMENU)
        .style(WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_OWNERDATA | LVS_ALIGNLEFT | LVS_SHAREIMAGELISTS | WS_CHILD)
        .h_parent(parent)
        .build();
    let list_view = wnd::Wnd::new(list_view_params)?;
    unsafe { SendMessageW(list_view.hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as WPARAM, (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as LPARAM); };
    let header = ListHeader::create(&list_view);
    Ok((list_view, header))
}

pub struct ItemList {
    wnd: Wnd,
    header: ListHeader,
    item_paint: ItemPaint,
    plugin: Arc<Plugin>,
}

impl ItemList {
    fn new(wnd: Wnd, header: ListHeader, plugin: Arc<Plugin>) -> ItemList {
        let item_paint = ItemPaint::create();
        ItemList {
            wnd,
            header,
            item_paint,
            plugin,
        }
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

    pub fn update(&self, state: &State) {
        self.scroll_to_top();
        self.wnd.send_message(LVM_SETITEMCOUNT, state.count() as WPARAM, 0);
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

    pub fn custom_draw(&mut self, event: Event, state: &State) -> LRESULT {
        let custom_draw = event.as_custom_draw();
        const SUBITEM_PAINT: u32 = CDDS_SUBITEM | CDDS_ITEMPREPAINT;
        match custom_draw.nmcd.dwDrawStage {
            CDDS_PREPAINT => {
                CDRF_NOTIFYITEMDRAW
            }
            CDDS_ITEMPREPAINT => {
                self.plugin.prepare_item(custom_draw.nmcd.dwItemSpec, state);
                CDRF_NOTIFYSUBITEMDRAW
            }
            SUBITEM_PAINT => {
                println!("prepaint item {} {}", custom_draw.nmcd.dwItemSpec, custom_draw.iSubItem);
                if custom_draw.iSubItem == 0 {
                    let rc = custom_draw.nmcd.rc;

                    println!("{} {}", rc.left, rc.right);
                    match self.plugin.get_draw_info(event, custom_draw.nmcd.dwItemSpec as usize, custom_draw.iSubItem) {
                        ItemDraw::DETAILED(m) => {
                            self.item_paint.draw_name(custom_draw, &m)
                        }
                        _ => unreachable!()
                    }
                    CDRF_SKIPDEFAULT
                } else {
                    CDRF_DODEFAULT
                }
            }
            _ => {
                CDRF_DODEFAULT
            }
        }
        //
    }

//    pub fn draw_item_old(&mut self, event: Event, _state: &State) {
//        let draw_item = event.as_draw_item();
//
//        match draw_item.itemAction {
//            ODA_DRAWENTIRE => {
//                let mut positions = [
//                    self.painting_position_of(draw_item, 0),
//                    self.painting_position_of(draw_item, 1),
//                    self.painting_position_of(draw_item, 2),
//                ];
//                self.plugin.draw_item(event, positions);
//            }
//            /*
//            if (Item->itemState & ODS_FOCUS)
//                {
//                    DrawFocusRect(Item->hDC, &Item->rcItem);
//                }
//                */
//            _ => panic!("other"),
//        }
//    }

    pub fn display_item(&mut self, event: Event, state: &State) {
        let item = &mut event.as_display_info().item;
        if (item.mask & LVIF_IMAGE) == LVIF_IMAGE {
            println!("IMAGE {} ", item.iSubItem);
        }
        if (item.mask & LVIF_TEXT) == LVIF_TEXT {
            match self.plugin.get_draw_info(event, item.iItem as usize, item.iSubItem) {
                ItemDraw::SIMPLE(txt) => {
                    item.pszText = txt;
                }
                _ => {}
            }
        }
    }
}

