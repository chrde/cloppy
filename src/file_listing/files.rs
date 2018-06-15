use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use plugin::ItemIdx;
use std::collections::HashMap;
use std::mem;
use std::time::Instant;
use twoway;

pub struct Files {
    separator: String,
    //    files: Vec<FileEntity>,
    files: HashMap<ItemIdx, FileEntity>,
    directories: HashMap<FileId, ItemIdx>,
}

unsafe impl Send for Files {}

impl Files {
    pub fn new(count: usize) -> Self {
        let files = HashMap::with_capacity(count);
        let directories = HashMap::with_capacity(count);
        let separator = "\\".to_owned();
        Files { files, directories, separator }
    }
    pub fn add_file(&mut self, f: FileEntity) {
        if f.is_directory() {
            self.directories.insert(f.id(), ItemIdx::new(self.files.len()));
        }
        let count = self.files.len();
        self.files.insert(ItemIdx::new(count), f);
    }

//    pub fn update_file(&mut self, file: FileEntity) {
//        let file_position = self.file_locations.get(&file.id()).unwrap().clone();
//        let old = self.files.get_mut(file_position.id()).unwrap();
//        assert_eq!(old.id(), file.id());
//        mem::swap(old, &mut file
//        self.files.repl
//    }

    pub fn file(&self, pos: ItemIdx) -> &FileEntity {
        self.files.get(&pos).unwrap()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

//    pub fn sort_by_name(&mut self) {
//        self.files.sort_unstable_by(FileEntity::name_comparator);
//        let mut data = Vec::with_capacity(self.files.capacity());
//        for f in &self.files {
//            self.directories.insert(f.id(), ItemIdx::new(data.len()));
//            data.push(f.clone())
//        }
//        mem::swap(&mut data, &mut self.files);
//    }

    pub fn path_of(&self, file: &FileEntity) -> String {
        let mut result = String::new();
        let mut parents: Vec<ItemIdx> = Vec::new();
        let mut current = file;
        while !current.is_root() {
            let parent_pos = self.directories.get(&current.parent_id()).expect(&format!("parent for {:?} not found", current.id()));
            let parent = self.files.get(parent_pos).unwrap();
            parents.push(parent_pos.clone());
            current = parent;
        }
        for p in parents.iter().rev() {
            result.push_str(self.files.get(p).map(|f| f.name()).unwrap());
            result.push_str(&self.separator);
        }
        result
    }

    pub fn search_by_name<'a, T>(&self, name: &'a str, items: T) -> Vec<ItemIdx>
        where T: IntoIterator<Item=ItemIdx> {
        let now = Instant::now();
        let mut result = Vec::new();
        for idx in items {
            let mut file_name = &self.files.get(&idx).unwrap().name();
            if twoway::find_str(file_name, name).is_some() {
                result.push(idx);
            }
        }
        println!("total time {:?}", Instant::now().duration_since(now).subsec_nanos() / 1_000_000);
        result
    }

    pub fn new_search_by_name<'a>(&self, name: &'a str) -> Vec<ItemIdx> {
        let items = (0..self.len()).into_iter().map(|x| ItemIdx::new(x));
        self.search_by_name(name, items)
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
        assert!(files.new_search_by_name("").is_empty())
    }

    #[test]
    fn search_by_name() {
        let mut files = Files::new(4);
        files.add_file(new_file("a"));
        files.add_file(new_file("ba"));
        files.add_file(new_file("baba"));
        files.add_file(new_file("b"));

        let results = files.new_search_by_name("a");
        assert_eq!(3, results.len());
        assert_eq!(&"a", &files.file(results.get(0).unwrap().clone()).name());
        assert_eq!(&"ba", &files.file(results.get(1).unwrap().clone()).name());
        assert_eq!(&"baba", &files.file(results.get(2).unwrap().clone()).name());

        let results = files.search_by_name("b", results);
        assert_eq!(2, results.len());
        assert_eq!(&"ba", &files.file(results.get(0).unwrap().clone()).name());
        assert_eq!(&"baba", &files.file(results.get(1).unwrap().clone()).name());
    }

    #[test]
    fn get_paths() {
        let mut files = Files::new(5);
        files.add_file(new_dir("d1", 0));
        files.add_file(new_dir("d2", 1));
        files.add_file(new_dir("d3", 2));
        files.add_file(new_file_with_parent("f1", 3, 0));
        files.add_file(new_file_with_parent("f2", 4, 1));
        files.add_file(new_file_with_parent("f3", 5, 2));
        files.add_file(new_file_with_parent("f4", 6, 2));

        let f = files.file(ItemIdx::new(3));
        assert_eq!("d1\\", files.path_of(f));
        let f = files.file(ItemIdx::new(4));
        assert_eq!("d1\\d2\\", files.path_of(f));
        let f = files.file(ItemIdx::new(5));
        assert_eq!("d1\\d3\\", files.path_of(f));
        let f = files.file(ItemIdx::new(6));
        assert_eq!("d1\\d3\\", files.path_of(f));
    }
}
