use file_listing::file_entity::FileId;
use file_listing::list::item::DisplayItem;
use plugin::PluginState;
use std::collections::HashMap;

#[derive(Default)]
pub struct FilesState {
    current_search: Vec<FileId>,
    item_cache: HashMap<u32, DisplayItem>,
}

impl FilesState {
    pub fn item_cache(&self) -> &hashmap<u32, displayitem> {
        &self.item_cache
    }

    pub fn current_search(&self) -> &[FileId] {
        &self.current_search
    }
}

impl PluginState for FilesState {}

