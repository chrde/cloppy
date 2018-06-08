use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use file_listing::list::item::*;
use std::collections::HashMap;
use std::mem;
use std::time::Instant;
use twoway;

#[derive(Clone, Debug)]
pub struct ItemIdx(usize);

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
            self.directories.insert(f.id(), ItemIdx(self.files.len()));
        }
        self.files.push(f);
    }

    pub fn file(&self, pos: ItemIdx, query: &str) -> DisplayItem {
        use windows::utils::ToWide;
        let file = self.files.get(pos.0).unwrap();
        let matches = matches(query, &file.name());
        DisplayItem {
            name: file.name().to_owned(),
            path: self.calculate_path_of(pos).to_wide_null(),
            size: file.size().to_string().to_wide_null(),
            matches,
            flags: file.flags(),
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn sort_by_name(&mut self) {
        self.files.sort_unstable_by(FileEntity::name_comparator);
        let mut data = Vec::with_capacity(self.files.capacity());
        for f in &self.files {
            self.directories.insert(f.id(), ItemIdx(data.len()));
            data.push(f.clone())
        }
        mem::swap(&mut data, &mut self.files);
    }

    fn calculate_path_of(&self, pos: ItemIdx) -> String {
        let mut result = String::new();
        let mut parents: Vec<ItemIdx> = Vec::new();
        let mut current = &self.files[pos.0];
        while !current.is_root() {
            let parent_pos = self.directories.get(&current.parent_id()).expect(&format!("parent for {:?} not found", current.id()));
            let parent = self.files.get(parent_pos.0).unwrap();
            parents.push(parent_pos.clone());
            current = parent;
        }
        for p in parents.into_iter().rev() {
            result.push_str(self.files.get(p.0).map(|f| f.name()).unwrap());
            result.push_str("\\");
        }
        result
    }

    pub fn search_by_name<'a, T>(&self, name: &'a str, items: T) -> Vec<ItemIdx>
        where T: IntoIterator<Item=ItemIdx> {
        let now = Instant::now();
        let mut result = Vec::new();
        for idx in items {
            let mut file_name = &self.files.get(idx.0).unwrap().name();
            if twoway::find_str(file_name, name).is_some() {
                result.push(idx);
            }
        }
        println!("total time {:?}", Instant::now().duration_since(now).subsec_nanos() / 1_000_000);
        result
    }

    pub fn new_search_by_name<'a>(&self, name: &'a str) -> Vec<ItemIdx> {
        let items = (0..self.file_count()).into_iter().map(|x| ItemIdx(x));
        self.search_by_name(name, items)
    }
}
