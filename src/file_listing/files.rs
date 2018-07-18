use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use file_listing::storage::Storage;
use file_listing::storage::StorageItem;
use std::borrow::Borrow;
use std::cmp::Ordering;
use twoway;

#[derive(Debug, Eq)]
pub struct FileData {
    id: FileId,
    parent_id: FileId,
    name_id: NameId,
    size: i64,
    flags: u16,
    deleted: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NameId(pub u32);

impl PartialOrd for FileData {
    fn partial_cmp(&self, other: &FileData) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl Ord for FileData {
    fn cmp(&self, other: &FileData) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialEq for FileData {
    fn eq(&self, other: &FileData) -> bool {
        self.id == other.id
    }
}

impl FileData {
    pub fn new(id: FileId, parent_id: FileId, size: i64, flags: u16, deleted: bool) -> FileData {
        FileData {
            id,
            parent_id,
            size,
            flags,
            deleted,
            name_id: NameId(0),
        }
    }

    pub fn set_name_id(&mut self, name_id: NameId) {
        self.name_id = name_id;
    }

    pub fn set_deleted(&mut self, deleted: bool) {
        self.deleted = deleted;
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn name_id(&self) -> NameId {
        self.name_id
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
            name_id: NameId(0),
        }
    }
}

pub struct Files {
    separator: String,
    storage: Storage,
}

unsafe impl Send for Files {}

impl Files {
    pub fn new(_count: usize) -> Self {
        let storage = Storage::new();
        let separator = "\\".to_owned();
        Files { storage, separator }
    }

    pub fn bulk_add(&mut self, files: Vec<FileEntity>) {
        self.storage.bulk_insert(files);
    }

    pub fn add_file(&mut self, f: FileEntity) {
        self.storage.upsert(f.clone().into(), f.name());
    }

    pub fn update_file(&mut self, f: FileEntity) {
        self.storage.upsert(f.clone().into(), f.name());
    }

    pub fn get_file<T: Borrow<FileId>>(&self, pos: T) -> StorageItem {
        self.storage.get(pos)
    }

    pub fn delete_file(&mut self, id: FileId) {
        self.storage.delete(id);
    }

    pub fn path_of(&self, file: &FileData) -> String {
        let mut result = String::new();
        let mut parents: Vec<&str> = Vec::new();
        let mut current = file;
        while !current.is_root() {
            let item = self.storage.get(current.parent_id());
            parents.push(item.name);
            current = item.data;
        }
        for p in parents.into_iter().rev() {
            result.push_str(p);
            result.push_str(&self.separator);
        }
        result
    }

//    fn new_search_by_name<'a>(&self, name: &'a str) -> Vec<ItemId> {
//        println!("2");
//        println!("total {}", self.storage.iter().count());
//        let now = Instant::now();
//        let total = self.storage.iter().filter(|item| twoway::find_str(item.name, name).is_some()).count();
//        println!("new search found {} in {:?}ms", total, Instant::now().duration_since(now));
//        self.names.par_iter().enumerate()
//            .filter(|(_, file_name)| twoway::find_str(file_name, name).is_some())
//            .map(|(pos, _)| FileId::new(pos as u32))
//            .collect()
//    }

//    fn continue_search_by_name<'a>(&self, name: &'a str, prev_search: &[ItemId]) -> Vec<ItemId> {
//        prev_search.par_iter().cloned()
//            .filter(|pos| twoway::find_str(self.get_name_of(*pos), name).is_some())
//            .collect()
//    }


    pub fn search_by_name<'a>(&self, name: &'a str, _prev_search: Option<&[FileId]>) -> Vec<FileId> {
        self.storage.iter()
            .filter(|item| twoway::find_str(item.name, name).is_some())
            .map(|i| i.data.id())
            .collect()
    }

//    pub fn search_by_name<'a>(&self, name: &'a str, prev_search: Option<&[FileId]>) -> Vec<FileId> {
//        if name.is_empty() {
//            println!("1");
//            self.sorted_idx.clone()
//        } else {
//            match prev_search {
//                None => self.new_search_by_name(name),
//                Some(prev) => self.continue_search_by_name(name, prev)
//            }
//        }
//    }
}

#[cfg(test)]
mod tests {
    use file_listing::file_entity::FileId;
    use ntfs::attributes::FilenameAttr;
    use ntfs::file_record::FileRecord;
    use super::*;

    const FILE: u16 = 1;
    const DIR: u16 = 2;

    fn test_data() -> Files {
        let mut files = Files::new(1);
        let file0 = FileData::new(FileId::file(0), FileId::directory(1), 0, FILE, false);
        let dir0 = FileData::new(FileId::directory(0), FileId::directory(1), 0, DIR, false);
        let file1 = FileData::new(FileId::file(1), FileId::directory(1), 0, FILE, false);
        let dir1 = FileData::new(FileId::directory(1), FileId::directory(1), 0, DIR, false);
        let file2 = FileData::new(FileId::file(2), FileId::directory(1), 0, FILE, false);
        let dir2 = FileData::new(FileId::directory(2), FileId::directory(1), 0, DIR, false);
        let dir3 = FileData::new(FileId::directory(3), FileId::directory(2), 0, DIR, false);

        files.storage.upsert(file2, "file2");
        files.storage.upsert(file1, "file1");
        files.storage.upsert(file0, "file0");
        files.storage.upsert(dir3, "dir3");
        files.storage.upsert(dir2, "dir2");
        files.storage.upsert(dir1, "dir1");
        files.storage.upsert(dir0, "dir0");

        files
    }

    fn new_file_record(name: &str) -> FileRecord {
        let mut file_record = FileRecord::default();
        let mut entry_name = FilenameAttr::default();
        entry_name.name = name.to_string();
        file_record.name_attrs = vec![entry_name];
        file_record
    }

    fn new_file(name: &str) -> FileEntity {
        FileEntity::from_file_entry(new_file_record(name))
    }

    fn new_dir(name: &str, id: u32) -> FileEntity {
        let mut entry = new_file_record(name);
        entry.header.flags = 0x02;
        entry.header.fr_number = id;
        FileEntity::from_file_entry(entry)
    }

    fn new_file_with_parent(name: &str, id: u32, parent: u32) -> FileEntity {
        let mut entry = new_file_record(name);
        entry.name_attrs[0].parent_id = parent as i64;
        entry.header.fr_number = id;
        FileEntity::from_file_entry(entry)
    }

    #[test]
    fn empty_files() {
        let files = Files::new(5);
        assert!(files.search_by_name("", None).is_empty())
    }

    #[test]
    fn search_by_name() {
        let files = test_data();

        let search = files.search_by_name("0", None);
        assert_eq!(2, search.len());
        assert_eq!(&"dir0", &files.get_file(search.get(0).unwrap()).name);
        assert_eq!(&"file0", &files.get_file(search.get(1).unwrap()).name);

        let search = files.search_by_name("4", None);
        assert!(search.is_empty());
    }

    #[test]
    fn get_paths() {
        let files = test_data();

        let f = files.get_file(FileId::file(0)).data;
        assert_eq!("dir1\\", files.path_of(f));
        let f = files.get_file(FileId::file(1)).data;
        assert_eq!("dir1\\", files.path_of(f));
        let f = files.get_file(FileId::directory(3)).data;
        assert_eq!("dir1\\dir2\\", files.path_of(f));
    }

    #[test]
    fn after_adding_file_sorted_new_file_is_present() {
        let mut files = test_data();
        let prev_search = files.search_by_name("file0", None).len();
        let new_file = FileData::new(FileId::file(3), FileId::directory(1), 42, FILE, false);

        files.storage.upsert(new_file, "a_file0");
        let search = files.search_by_name("file0", None);

        assert_eq!(prev_search + 1, search.len());
        assert_eq!(&"a_file0", &files.get_file(search.get(1).unwrap()).name);
    }

    #[test]
    fn adding_file_doesnt_invalidate_existing_item_id() {
        let mut files = test_data();

        let search = files.search_by_name("file0", None);
        let new_file = FileData::new(FileId::file(3), FileId::directory(1), 42, FILE, false);
        files.storage.upsert(new_file, "a_file0");

        assert_eq!(1, search.len());
        assert_eq!(&"file0", &files.get_file(search.get(0).unwrap()).name);
    }

    #[test]
    fn update_existing_file() {
        let mut files = test_data();
        let update_file = FileData::new(FileId::file(0), FileId::directory(1), 42, FILE, false);
        files.storage.upsert(update_file, "new_name");

        assert!(files.search_by_name(&"file0", None).is_empty());
        let search = files.search_by_name(&"new_name", None);
        assert_eq!(1, search.len());
        assert_eq!(FileId::file(0), files.get_file(search[0]).data.id());
        assert_eq!("new_name", files.get_file(search[0]).name);
    }
}

