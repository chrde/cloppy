use errors::failure_to_string;
use file_listing::files::Files;
use file_listing::list::item::DisplayItem;
use file_listing::list::paint::ItemPaint;
use file_listing::ntfs::change_journal;
use gui::event::Event;
use plugin::CustomDrawResult;
use plugin::DrawResult;
use plugin::ItemIdx;
use plugin::Plugin;
use plugin::State;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::RwLock;

mod list;
mod ntfs;
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
    pub fn create(files: Files) -> Self {
        let items_cache = HashMap::new();
        let item_paint = ItemPaint::create();
        let (sender, receiver) = mpsc::channel();
        change_journal::run(sender).unwrap();
        change_journal::debug(receiver).unwrap();
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
    fn draw_item(&self, event: Event) -> DrawResult {
        let inner = self.0.read().unwrap();
        inner.item_paint.draw_item(event, &inner.items_cache)
    }

    fn custom_draw_item(&self, event: Event) -> CustomDrawResult {
        let inner = self.0.read().unwrap();
        inner.item_paint.custom_draw_item(event, &inner.items_cache)
    }

    fn prepare_item(&self, item_id: usize, state: &State) {
        let inner: &mut Inner = &mut *self.0.write().unwrap();
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
            let inner: &mut Inner = &mut *self.0.write().unwrap();
            inner.last_search = msg.clone();
            inner.items_current_search = items.clone();
        }
        Box::new(State::new(msg, items))
    }

    fn start(&self) {}
}