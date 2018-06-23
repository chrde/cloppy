use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use plugin::ItemId;
use rayon::prelude::*;
use std::collections::HashMap;
use std::mem;
use std::time::Instant;
use twoway;

pub struct Files {
    separator: String,
    files: Vec<FileData>,
    names: Vec<String>,
    sorted_idx: Vec<ItemId>,
    file_id_idx: HashMap<FileId, ItemId>,
}

#[derive(Debug)]
pub struct FileData {
    parent_id: FileId,
    size: i64,
    id: FileId,
    flags: u16,
    deleted: bool,
}

impl FileData {
    pub fn set_deleted(&mut self, deleted: bool) {
        self.deleted = deleted;
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn parent_id(&self) -> FileId {
        self.parent_id
    }

    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }
    pub fn is_root(&self) -> bool {
        self.parent_id == self.id
    }

    pub fn is_directory(&self) -> bool {
        self.flags & 0x02 != 0
    }
}

impl From<FileEntity> for FileData {
    fn from(f: FileEntity) -> FileData {
        FileData {
            parent_id: f.parent_id(),
            size: f.size(),
            id: f.id(),
            flags: f.flags(),
            deleted: false,
        }
    }
}

unsafe impl Send for Files {}

impl Files {
    pub fn new(count: usize) -> Self {
        let files = Vec::with_capacity(count);
        let sorted_idx = Vec::with_capacity(count);
        let names = Vec::new();
        let file_id_idx = HashMap::with_capacity(count);
        let separator = "\\".to_owned();
        Files { files, sorted_idx, names, file_id_idx, separator }
    }

    pub fn bulk_add(&mut self, files: Vec<FileEntity>) {
        for f in files {
            self.add_file(f, None);
        }
        self.sort_by_name();
    }

    fn add_file(&mut self, f: FileEntity, sorted_pos: Option<usize>) {
        let id = ItemId::new(self.files.len());
        self.file_id_idx.insert(f.id(), id);
        self.sorted_idx.insert(sorted_pos.unwrap_or(self.files.len()), id);
        self.names.push(f.name().to_string());
        self.files.push(f.into());
    }

    pub fn update_file(&mut self, file: FileEntity) {
        match self.file_id_idx.get(&file.id()).cloned() {
            None => {
                println!("update file - old not found - doing insert instead\n\tnew:\t {:?}", file);
                self.add_file_sorted_by_name(file);
            }
            Some(id) => {
                self.names[id.id()] = file.name().to_string();
                let old = self.get_file_mut(id);
                let new = file.into();
                println!("update file \n\t old:\t {:?}\n\tnew:\t {:?}", old, new);
                mem::replace(old, new);
            }
        }
    }

    pub fn add_file_sorted_by_name(&mut self, file: FileEntity) {
        println!("add file\n\t {:?}", file);
        let pos = match self.sorted_idx.binary_search_by(|id| {
            let cur = self.get_name_of(*id);
            cur.cmp(&file.name())
        }) {
            Ok(pos) => pos,
            Err(pos) => pos,
        };
        self.add_file(file, Some(pos));
    }

    fn get_file_mut(&mut self, pos: ItemId) -> &mut FileData {
        self.files.get_mut(pos.id()).unwrap()
    }

    pub fn get_file(&self, pos: ItemId) -> &FileData {
        self.files.get(pos.id()).unwrap()
    }

    pub fn get_name_of(&self, pos: ItemId) -> &str {
        self.names.get(pos.id()).unwrap()
    }

    pub fn delete_file(&mut self, id: FileId) {
        println!("delete file");
        if let Some(id) = self.file_id_idx.get(&id).cloned() {
            println!("Delete file\tOk\t{:?}", id);
            self.get_file_mut(id).set_deleted(true);
        } else {
            println!("Delete file\tNot found\t{:?}", id);
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn sort_by_name(&mut self) {
        let now = Instant::now();
        let names = &self.names;
        self.sorted_idx.sort_unstable_by_key(|pos| names.get(pos.id()).unwrap());
        println!("sort by name - total time {:?}", Instant::now().duration_since(now));
    }

    pub fn path_of(&self, file: &FileData) -> String {
        let mut result = String::new();
        let mut parents: Vec<ItemId> = Vec::new();
        let mut current = file;
        while !current.is_root() {
            let parent_pos = self.file_id_idx.get(&current.parent_id()).expect(&format!("parent for {:?} not found", current.id()));
            let parent = self.get_file(parent_pos.clone());
            parents.push(parent_pos.clone());
            current = parent;
        }
        for p in parents.into_iter().rev() {
            result.push_str(&self.get_name_of(p));
            result.push_str(&self.separator);
        }
        result
    }

    fn new_search_by_name<'a>(&self, name: &'a str) -> Vec<ItemId> {
        self.names.par_iter().enumerate()
            .filter(|(_, file_name)| twoway::find_str(file_name, name).is_some())
            .map(|(pos, _)| ItemId::new(pos))
            .collect()
    }

    fn continue_search_by_name<'a>(&self, name: &'a str, prev_search: &[ItemId]) -> Vec<ItemId> {
        prev_search.par_iter().cloned()
            .filter(|pos| twoway::find_str(self.get_name_of(*pos), name).is_some())
            .collect()
    }

    pub fn search_by_name<'a>(&self, name: &'a str, prev_search: Option<&[ItemId]>) -> Vec<ItemId> {
        if name.is_empty() {
            self.sorted_idx.clone()
        } else {
            match prev_search {
                None => self.new_search_by_name(name),
                Some(prev) => self.continue_search_by_name(name, prev)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ntfs::file_entry::FileEntry;
    use ntfs::file_entry::FileEntryName;
    use super::*;

    fn new_file_entry(name: &str) -> FileEntry {
        let mut file_entry = FileEntry::default();
        let mut entry_name = FileEntryName::default();
        entry_name.name = name.to_string();
        file_entry.names = vec![entry_name];
        file_entry
    }

    fn new_file(name: &str) -> FileEntity {
        FileEntity::from_file_entry(new_file_entry(name))
    }

    fn new_dir(name: &str, id: u32) -> FileEntity {
        let mut entry = new_file_entry(name);
        entry.flags = 0x02;
        entry.id = id;
        FileEntity::from_file_entry(entry)
    }

    fn new_file_with_parent(name: &str, id: u32, parent: u32) -> FileEntity {
        let mut entry = new_file_entry(name);
        entry.names[0].parent_id = parent;
        entry.id = id;
        FileEntity::from_file_entry(entry)
    }

    #[test]
    fn empty_files() {
        let files = Files::new(5);
        assert_eq!(0, files.len());
        assert!(files.search_by_name("", None).is_empty())
    }

    #[test]
    fn search_by_name() {
        let mut files = Files::new(4);
        files.add_file(new_file("a"), None);
        files.add_file(new_file("ba"), None);
        files.add_file(new_file("baba"), None);
        files.add_file(new_file("b"), None);

        let search = files.search_by_name("a", None);
        assert_eq!(3, search.len());
        assert_eq!(&"a", &files.get_name_of(search.get(0).unwrap().clone()));
        assert_eq!(&"ba", &files.get_name_of(search.get(1).unwrap().clone()));
        assert_eq!(&"baba", &files.get_name_of(search.get(2).unwrap().clone()));

        let search = files.search_by_name("b", Some(&search));
        assert_eq!(2, search.len());
        assert_eq!(&"ba", &files.get_name_of(search.get(0).unwrap().clone()));
        assert_eq!(&"baba", &files.get_name_of(search.get(1).unwrap().clone()));
    }

    #[test]
    fn get_paths() {
        let mut files = Files::new(5);
        files.add_file(new_dir("d1", 0), None);
        files.add_file(new_dir("d2", 1), None);
        files.add_file(new_dir("d3", 2), None);
        files.add_file(new_file_with_parent("f1", 3, 0), None);
        files.add_file(new_file_with_parent("f2", 4, 1), None);
        files.add_file(new_file_with_parent("f3", 5, 2), None);
        files.add_file(new_file_with_parent("f4", 6, 2), None);

        let f = files.get_file(ItemId::new(3));
        assert_eq!("d1\\", files.path_of(f));
        let f = files.get_file(ItemId::new(4));
        assert_eq!("d1\\d2\\", files.path_of(f));
        let f = files.get_file(ItemId::new(5));
        assert_eq!("d1\\d3\\", files.path_of(f));
        let f = files.get_file(ItemId::new(6));
        assert_eq!("d1\\d3\\", files.path_of(f));
    }

    #[test]
    fn after_adding_file_sorted_new_file_is_present() {
        let mut files = Files::new(3);
        files.add_file(new_file("a"), None);
        files.add_file(new_file("b"), None);

        files.add_file_sorted_by_name(new_file("aa"));
        let search = files.search_by_name("aa", None);
        assert_eq!(1, search.len());
        assert_eq!(&"aa", &files.get_name_of(search.get(0).unwrap().clone()));
    }

    #[test]
    fn adding_file_doesnt_invalidate_existing_item_id() {
        let mut files = Files::new(3);
        files.add_file(new_file("a"), None);
        files.add_file(new_file("aba"), None);
        files.add_file(new_file("b"), None);

        let search = files.search_by_name("aba", None);
        files.add_file_sorted_by_name(new_file("aa"));

        assert_eq!(1, search.len());
        assert_eq!(&"aba", &files.get_name_of(search.get(0).unwrap().clone()));
    }

    #[test]
    fn update_existing_file() {
        let mut files = Files::new(1);
        let mut old = new_file_entry("old");
        old.id = 1;
        let mut new = new_file_entry("new");
        new.id = 1;
        files.add_file(FileEntity::from_file_entry(old), None);

        files.update_file(FileEntity::from_file_entry(new));

        assert!(files.search_by_name(&"old", None).is_empty());
        let search = files.search_by_name(&"new", None);
        assert_eq!(1, search.len());
        assert_eq!(FileId::new(1), files.get_file(search[0]).id());
        assert_eq!("new", files.get_name_of(search[0]));
    }

    #[test]
    fn update_non_existing_file() {
        let mut files = Files::new(0);
        let mut new = new_file_entry("new");
        new.id = 1;

        files.update_file(FileEntity::from_file_entry(new));

        let search = files.search_by_name(&"new", None);
        assert_eq!(1, search.len());
        assert_eq!(FileId::new(1), files.get_file(search[0]).id());
        assert_eq!("new", files.get_name_of(search[0]));
    }
}

