use file_listing::files::Files;
use file_listing::list::item::DisplayItem;
use file_listing::list::paint::ItemPaint;
use gui::event::Event;
use plugin::ItemDraw;
use plugin::ItemIdx;
use plugin::Plugin;
use plugin::State;
use std::collections::HashMap;
use std::sync::RwLock;
use winapi::shared::minwindef::LRESULT;
use winapi::um::commctrl::CDRF_DODEFAULT;
use winapi::um::commctrl::CDRF_SKIPDEFAULT;
use winapi::um::winnt::LPWSTR;

mod list;
pub mod file_entity;
pub mod files;

pub struct FileListing(RwLock<Inner>);

struct Inner {
    last_search: String,
    files: Files,
    items_current_search: Vec<ItemIdx>,
    items_cache: HashMap<u32, DisplayItem>,
    item_paint: ItemPaint,
}

unsafe impl Sync for Inner {}

impl FileListing {
    pub fn new(files: Files) -> Self {
        let items_cache = HashMap::new();
        let item_paint = ItemPaint::create();
        let inner = Inner {
            files,
            item_paint,
            items_cache,
            last_search: String::new(),
            items_current_search: Vec::new(),
        };
        let res = RwLock::new(inner);
        FileListing(res)
    }
}

impl Plugin for FileListing {
    fn draw_item(&self, file: usize, column: i32) -> ItemDraw {
        let inner = self.0.read().unwrap();
        let item = inner.items_cache.get(&(file as u32)).unwrap();
        match column {
            0 => ItemDraw::IGNORE,
            1 => ItemDraw::SIMPLE(item.path.as_ptr() as LPWSTR),
            2 => ItemDraw::SIMPLE(item.size.as_ptr() as LPWSTR),
            _ => unreachable!()
        }
    }

    fn custom_draw_item(&self, event: Event) -> LRESULT {
        let inner = self.0.read().unwrap();
        let custom_draw = event.as_custom_draw();
        if custom_draw.iSubItem == 0 {
            let item = inner.items_cache.get(&(custom_draw.nmcd.dwItemSpec as u32)).unwrap();
            inner.item_paint.draw_name(custom_draw, &item.matches);
            CDRF_SKIPDEFAULT
        } else {
            CDRF_DODEFAULT
        }
    }

    fn prepare_item(&self, item_id: usize, state: &State) {
        let inner = &mut *self.0.write().unwrap();
        let position = state.items()[item_id].clone();
        let file = inner.files.file(position);
        let path = inner.files.path_of(file);
        inner.items_cache.insert(item_id as u32, DisplayItem::new(file, path, &state.query()));
    }

    fn handle_message(&self, msg: String) -> Box<State> {
        let items = {
            let inner = self.0.read().unwrap();
            if msg.starts_with(&inner.last_search) {
                inner.files.search_by_name(&msg, inner.items_current_search.iter().cloned())
            } else {
                inner.files.new_search_by_name(&msg)
            }
        };
        {
            let mut inner = self.0.write().unwrap();
            inner.last_search = msg.clone();
            inner.items_current_search = items.clone();
        }
        Box::new(State::new(msg, items))
    }
}