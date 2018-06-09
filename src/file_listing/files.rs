use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use plugin::ItemIdx;
use std::collections::HashMap;
use std::mem;
use std::time::Instant;
use twoway;

pub struct Files {
    files: Vec<FileEntity>,
    directories: HashMap<FileId, ItemIdx>,
}

unsafe impl Send for Files {}

impl Files {
    pub fn new(count: usize) -> Self {
        let files = Vec::with_capacity(count);
        let directories = HashMap::with_capacity(count);
        Files { files, directories }
    }
    pub fn add_file(&mut self, f: FileEntity) {
        if f.is_directory() {
            self.directories.insert(f.id(), ItemIdx::new(self.files.len()));
        }
        self.files.push(f);
    }

    pub fn file(&self, pos: ItemIdx) -> &FileEntity {
        self.files.get(pos.id()).unwrap()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn sort_by_name(&mut self) {
        self.files.sort_unstable_by(FileEntity::name_comparator);
        let mut data = Vec::with_capacity(self.files.capacity());
        for f in &self.files {
            self.directories.insert(f.id(), ItemIdx::new(data.len()));
            data.push(f.clone())
        }
        mem::swap(&mut data, &mut self.files);
    }

    pub fn path_of(&self, file: &FileEntity) -> String {
        let mut result = String::new();
        let mut parents: Vec<ItemIdx> = Vec::new();
        let mut current = file;
        while !current.is_root() {
            let parent_pos = self.directories.get(&current.parent_id()).expect(&format!("parent for {:?} not found", current.id()));
            let parent = self.files.get(parent_pos.id()).unwrap();
            parents.push(parent_pos.clone());
            current = parent;
        }
        for p in parents.into_iter().rev() {
            result.push_str(self.files.get(p.id()).map(|f| f.name()).unwrap());
            result.push_str("\\");
        }
        result
    }

    pub fn search_by_name<'a, T>(&self, name: &'a str, items: T) -> Vec<ItemIdx>
        where T: IntoIterator<Item=ItemIdx> {
        let now = Instant::now();
        let mut result = Vec::new();
        for idx in items {
            let mut file_name = &self.files.get(idx.id()).unwrap().name();
            if twoway::find_str(file_name, name).is_some() {
                result.push(idx);
            }
        }
        println!("total time {:?}", Instant::now().duration_since(now).subsec_nanos() / 1_000_000);
        result
    }

    pub fn new_search_by_name<'a>(&self, name: &'a str) -> Vec<ItemIdx> {
        let items = (0..self.file_count()).into_iter().map(|x| ItemIdx::new(x));
        self.search_by_name(name, items)
    }
}
