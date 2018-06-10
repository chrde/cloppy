use file_listing::files::Files;
use file_listing::list::item::DisplayItem;
use file_listing::list::paint::ItemPaint;
use gui::event::Event;
use gui::get_string_mut;
use plugin::ItemDraw;
use plugin::ItemIdx;
use plugin::Plugin;
use plugin::State;
use plugin::SuperMatch;
use std::ptr;
use std::sync::RwLock;
use winapi::shared::windef::RECT;
use winapi::um::winnt::LPWSTR;

pub mod list;
pub mod file_entity;
pub mod files;

pub struct FileListing(RwLock<Inner>);

struct Inner {
    last_query: String,
    files: Files,
    items_current_search: Vec<ItemIdx>,
    item_paint: ItemPaint,
}

unsafe impl Sync for Inner {}

impl FileListing {
    pub fn new(files: Files) -> Self {
        let item_paint = ItemPaint::create();
        let inner = Inner {
            files,
            last_query: String::new(),
            items_current_search: Vec::new(),
            item_paint,
        };
        let res = RwLock::new(inner);
        FileListing(res)
    }
    fn build_matches(&self, item: &DisplayItem) -> Vec<SuperMatch> {
        let mut result = Vec::with_capacity(item.matches.len());
        for m in &item.matches {
            result.push(SuperMatch {
                matched: m.matched,
                init: m.init,
                end: m.end,
                text: item.name[m.init..m.end].encode_utf16().collect(),
            })
        };
        result
    }
}

impl Plugin for FileListing {
    fn get_draw_info(&self, event: Event, file: usize, column: i32) -> ItemDraw {
        let inner = self.0.read().unwrap();
        let item = inner.item_paint.get_item(file as u32);
        match column {
            0 => ItemDraw::DETAILED(self.build_matches(item)),
            1 => ItemDraw::SIMPLE(item.path.as_ptr() as LPWSTR),
            2 => ItemDraw::SIMPLE(item.size.as_ptr() as LPWSTR),
            _ => unreachable!()
        }
    }

//    fn draw_item_name(&self, event: Event, file: usize, position: RECT) {
//        let inner = self.0.read().unwrap();
//        inner.item_paint.draw_name(event.as_custom_draw(), position);
//    }

    fn prepare_item(&self, item_id: usize, state: &State) {
        {
            let inner = &mut *self.0.write().unwrap();
            inner.item_paint.prepare_item(item_id, &inner.files, state);
        }
    }

    fn handle_message(&self, msg: String) -> Box<State> {
        let items = {
            let inner = self.0.read().unwrap();
            if msg.starts_with(&inner.last_query) {
                inner.files.search_by_name(&msg, inner.items_current_search.iter().cloned())
            } else {
                inner.files.new_search_by_name(&msg)
            }
        };
        {
            let mut inner = self.0.write().unwrap();
            inner.last_query = msg.clone();
            inner.items_current_search = items.clone();
        }
        Box::new(State::new(msg, items))
    }
}