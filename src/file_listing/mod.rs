use crossbeam_channel as channel;
use dispatcher::UiAsyncMessage;
use failure::Error;
use file_listing::files::Files;
use file_listing::FilesMsg::ChangeJournal;
use file_listing::list::item::DisplayItem;
use file_listing::list::paint::ItemPaint;
use file_listing::state::FilesState;
use gui::event::Event;
use ntfs::change_journal;
use ntfs::change_journal::UsnChange;
use plugin::CustomDrawResult;
use plugin::DrawResult;
use plugin::Plugin;
use plugin::PluginState;
use plugin::State;
use slog::Logger;
use std::sync::RwLock;
use std::thread;
use std::time::Instant;

mod list;
mod storage;
mod state;
pub mod file_entity;
pub mod files;

pub struct FileListing(RwLock<Inner>);

struct Inner {
    logger: Logger,
    files: Files,
    item_paint: ItemPaint,
}

unsafe impl Sync for Inner {}

impl FileListing {
    pub fn create(files: Files, sender: channel::Sender<UiAsyncMessage>, parent_logger: &Logger) -> Self {
        let logger = parent_logger.new(o!("plugin" =>"files"));
        let item_paint = ItemPaint::create();
        run_change_journal(sender).unwrap();
        let inner = Inner {
            files,
            logger,
            item_paint,
        };
        let res = RwLock::new(inner);
        FileListing(res)
    }

    pub fn on_message(&self, msg: FilesMsg) {
        match msg {
            ChangeJournal(changes) => self.update_files(changes),
        }
    }

    fn update_files(&self, changes: Vec<UsnChange>) {
        let inner: &mut Inner = &mut *self.0.write().unwrap();
        for change in changes {
            match change {
                UsnChange::DELETE(id) => inner.files.delete_file(id),
                UsnChange::UPDATE(file) => inner.files.update_file(file),
                UsnChange::NEW(file) => inner.files.add_file(file),
                UsnChange::IGNORE => {}
            }
        }
    }
}

pub enum FilesMsg {
    ChangeJournal(Vec<UsnChange>),
}

impl Plugin for FileListing {
    fn draw_item(&self, event: Event, state: &State) -> DrawResult {
        let inner = self.0.read().unwrap();
        let state = state.plugin_state::<FilesState>().unwrap();
        inner.item_paint.draw_item(event, state.item_cache())
    }

    fn custom_draw_item(&self, event: Event, state: &State) -> CustomDrawResult {
        let inner = self.0.read().unwrap();
        let state = state.plugin_state::<FilesState>().unwrap();
        inner.item_paint.custom_draw_item(event, state.item_cache())
    }

    fn prepare_item(&self, item_id: usize, state: &mut State) {
        let query = state.query().to_string();
        let inner = self.0.read().unwrap();
        let plugin_state = state.plugin_state_mut::<FilesState>().unwrap();
        let file = plugin_state.file_in_current_search(item_id)
            .map(|file_id| inner.files.get_file(file_id))
            .unwrap();
        let path = inner.files.path_of(file.data);
        plugin_state.item_cache_mut().insert(item_id as u32, DisplayItem::new(file.data, file.name.to_string(), path, &query));
    }

    fn handle_message(&self, msg: &str, _prev_state: &State) -> State {
        let now = Instant::now();
        let inner = self.0.read().unwrap();
        let items = {
//            if !inner.last_search.is_empty() && msg.starts_with(&inner.last_search) {
//                inner.files.search_by_name(&msg, Some(&inner.items_current_search))
//            } else {
            inner.files.search_by_name(msg, None)
//            }
        };
        let count = items.len();
        let files_state = Box::new(FilesState::new(items));
        info!(inner.logger, "handle_message"; "query" => msg, "time(ms)" => millis_since(now));
        State::new(msg, count, files_state)
    }

    fn default_plugin_state(&self) -> Box<PluginState> {
        Box::new(FilesState::default())
    }
}

fn millis_since(before: Instant) -> u32 {
    let now = Instant::now().duration_since(before);
    now.as_secs() as u32 * 1000 + now.subsec_millis()
}

pub fn run_change_journal(sender: channel::Sender<UiAsyncMessage>) -> Result<(), Error> {
    thread::Builder::new().name("read journal".to_string()).spawn(move || {
        let volume_path = "\\\\.\\C:";
        let mut journal = change_journal::UsnJournal::new(volume_path).unwrap();
        loop {
            let changes = journal.get_new_changes().unwrap();
            sender.send(UiAsyncMessage::Files(FilesMsg::ChangeJournal(changes)));
        }
    })?;
    Ok(())
}