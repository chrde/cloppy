use file_listing::file_entity::FileId;
use file_listing::list::item::DisplayItem;
use plugin::PluginState;
use std::any::Any;
use std::collections::HashMap;

#[derive(Default)]
pub struct FilesState {
    current_search: Vec<FileId>,
    item_cache: HashMap<u32, DisplayItem>,
}

impl FilesState {

    pub fn new(current_search: Vec<FileId>) -> FilesState {
        FilesState {
            current_search,
            item_cache: HashMap::new(),
        }
    }

    pub fn item_cache(&self) -> &HashMap<u32, DisplayItem> {
        &self.item_cache
    }

    pub fn item_cache_mut(&mut self) -> &mut HashMap<u32, DisplayItem> {
        &mut self.item_cache
    }

    pub fn file_in_current_search(&self, pos: usize) -> Option<&FileId> {
        self.current_search.get(pos)
    }
}

impl Clone for FilesState {
    fn clone(&self) -> Self {
        FilesState::new(self.current_search.clone())
    }
}

impl PluginState for FilesState {
    fn any_ref(&self) -> &Any {
        self
    }

    fn any_mut(&mut self) -> &mut Any {
        self
    }
}

