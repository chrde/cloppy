use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use file_listing::list::item::*;
use std::collections::HashMap;
use std::mem;
use std::time::Instant;
use twoway;

#[derive(Clone, Debug)]
pub struct FilePos(usize);

pub struct Arena {
    files: Vec<FileEntity>,
    directories: HashMap<FileId, FilePos>,
}

unsafe impl Send for Arena {}

impl Arena {
    pub fn new(count: usize) -> Self {
        let files = Vec::with_capacity(count);
        let parents = HashMap::with_capacity(count);
        Arena { files, directories: parents }
    }
    pub fn add_file(&mut self, f: FileEntity) {
        if f.is_directory() {
            self.directories.insert(f.id(), FilePos(self.files.len()));
        }
        self.files.push(f);
    }

    pub fn file(&self, pos: FilePos, query: &str) -> DisplayItem {
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
            self.directories.insert(f.id(), FilePos(data.len()));
            data.push(f.clone())
        }
        mem::swap(&mut data, &mut self.files);
    }

    pub fn set_paths(&self) {
        for id in 0..self.files.len() {
            let len = self.calculate_path_of(FilePos(id)).len();
            if len == 0 {
                println!("{} has no path", id);
            }
        }
    }

    fn calculate_path_of(&self, pos: FilePos) -> String {
        let mut result = String::new();
        let mut parents: Vec<FilePos> = Vec::new();
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

    pub fn search_by_name<'a, T>(&self, name: &'a str, items: T) -> Vec<FilePos>
        where T: IntoIterator<Item=FilePos> {
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

    pub fn new_search_by_name<'a>(&self, name: &'a str) -> Vec<FilePos> {
        let items = (0..self.file_count()).into_iter().map(|x| FilePos(x));
        self.search_by_name(name, items)
    }
}
