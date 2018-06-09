use file_listing::files::Files;
use file_listing::list::paint::ItemPaint;
use gui::event::Event;
use plugin::ItemIdx;
use plugin::Plugin;
use plugin::State;
use std::sync::RwLock;
use winapi::shared::windef::RECT;

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
}

impl Plugin for FileListing {
    fn draw_item(&self, event: Event, positions: [RECT; 3]) {
        let inner = self.0.read().unwrap();
        inner.item_paint.draw_item(event, positions);
    }

    fn prepare_item(&self, event: Event, state: &State) {
        let item = &mut event.as_display_info().item;
        {
            let inner = &mut *self.0.write().unwrap();
            inner.item_paint.prepare_item(item.iItem as u32, &inner.files, state);
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