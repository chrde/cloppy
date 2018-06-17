use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use plugin::ItemId;
use std::collections::HashMap;
use std::mem;
use std::time::Instant;
use twoway;

pub struct Files {
    separator: String,
    files: Vec<FileEntity>,
    sorted_idx: Vec<ItemId>,
    file_id_idx: HashMap<FileId, ItemId>,
}

unsafe impl Send for Files {}

impl Files {
    pub fn new(count: usize) -> Self {
        let files = Vec::with_capacity(count);
        let sorted_idx = Vec::with_capacity(count);
        let file_id_idx = HashMap::with_capacity(count);
        let separator = "\\".to_owned();
        Files { files, sorted_idx, file_id_idx, separator }
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
        self.files.push(f);
    }

    pub fn update_file(&mut self, file: FileEntity) {
        match self.file_id_idx.get(&file.id()).cloned() {
            None => {
                println!("update file - old not found - doing insert instead\n\tnew:\t {:?}", file);
                self.add_file_sorted_by_name(file);
            },
            Some(id) => {
                let old = self.get_file_mut(id);
                println!("update file \n\t old:\t {:?}\n\tnew:\t {:?}", old, file);
                mem::replace(old, file);
            }
        }
    }

    pub fn add_file_sorted_by_name(&mut self, file: FileEntity) {
        println!("add file\n\t {:?}", file);
        let pos = match self.sorted_idx.binary_search_by(|id| {
            let cur = self.get_file(*id).name();
            cur.cmp(&file.name())
        }) {
            Ok(pos) => pos,
            Err(pos) => pos,
        };
        self.add_file(file, Some(pos));
    }

    fn get_file_mut(&mut self, pos: ItemId) -> &mut FileEntity {
        self.files.get_mut(pos.file_pos()).unwrap()
    }

    pub fn get_file(&self, pos: ItemId) -> &FileEntity {
        self.files.get(pos.file_pos()).unwrap()
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
        let files = &self.files;
        self.sorted_idx.sort_unstable_by_key(|pos| files.get(pos.file_pos()).unwrap().name());
        println!("sort by name - total time {:?}", Instant::now().duration_since(now));
    }

    pub fn path_of(&self, file: &FileEntity) -> String {
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
            result.push_str(self.get_file(p).name());
            result.push_str(&self.separator);
        }
        result
    }

    pub fn search_by_name<'a>(&self, name: &'a str, prev_search: Option<&[ItemId]>) -> Vec<ItemId> {
        let now = Instant::now();
        let mut result = Vec::new();
        for pos in prev_search.unwrap_or(&self.sorted_idx) {
            let file = self.get_file(*pos);
            if !file.deleted() && twoway::find_str(file.name(), name).is_some() {
                result.push(*pos);
            }
        }
        println!("search total time {:?}", Instant::now().duration_since(now).subsec_nanos() / 1_000_000);
        result
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

        let results = files.search_by_name("a", None);
        assert_eq!(3, results.len());
        assert_eq!(&"a", &files.get_file(results.get(0).unwrap().clone()).name());
        assert_eq!(&"ba", &files.get_file(results.get(1).unwrap().clone()).name());
        assert_eq!(&"baba", &files.get_file(results.get(2).unwrap().clone()).name());

        let results = files.search_by_name("b", Some(&results));
        assert_eq!(2, results.len());
        assert_eq!(&"ba", &files.get_file(results.get(0).unwrap().clone()).name());
        assert_eq!(&"baba", &files.get_file(results.get(1).unwrap().clone()).name());
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
        assert_eq!(&"aa", &files.get_file(search.get(0).unwrap().clone()).name());
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
        assert_eq!(&"aba", &files.get_file(search.get(0).unwrap().clone()).name());
    }

    #[test]
    fn delete_file() {
        let mut files = Files::new(2);
        let mut file = new_file_entry("a");
        file.id = 1;
        files.add_file(FileEntity::from_file_entry(file), None);
        files.add_file(new_file("b"), None);

        files.delete_file(FileId(1));

        let search = files.search_by_name("a", None);
        assert_eq!(0, search.len());
    }
}

