use sql::FileEntity;
use std::time::Instant;
use twoway;
use std::collections::HashMap;
use std::mem;

#[derive(Default, Clone, Debug)]
pub struct ArenaFile {
    id: FileId,
    name: String,
    parent: FileId,
    flags: u8,
    size: i64,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq, Hash)]
struct FileId(usize);

#[derive(Clone, Debug)]
struct FilePos(usize);

impl ArenaFile {
    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub fn is_root(&self) -> bool {
        self.parent == self.id
    }

    pub fn is_directory(&self) -> bool {
        self.flags & 0x02 != 0
    }
}

pub struct Arena {
    files: Vec<ArenaFile>,
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
        let file = ArenaFile {
            id: FileId(f.id),
            name: f.name,
            parent: FileId(f.parent_id),
            size: f.size,
            flags: f.flags,
        };
        if file.is_directory() {
            self.directories.insert(file.id, FilePos(self.files.len()));
        }
        self.files.push(file);
    }

    fn get_file(&self, pos: FilePos) -> Option<&ArenaFile> {
        self.files.get(pos.0)
    }

    pub fn file(&self, idx: usize) -> Option<&ArenaFile> {
        self.get_file(FilePos(idx))
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn sort_by_name(&mut self) {
        self.files.sort_unstable_by(|x, y| x.name.cmp(&y.name));
        let mut data = Vec::with_capacity(self.files.capacity());
        for f in &self.files {
            self.directories.insert(f.id, FilePos(data.len()));
            data.push(f.clone())
        }
        mem::swap(&mut data, &mut self.files);
    }

    pub fn path_of(&self, idx: usize) -> String {
//        "".to_string()
        self.calculate_path_of(FilePos(idx))
    }

    pub fn set_paths(&self) {
        for id in 0..self.files.len() {
            let len = self.calculate_path_of(FilePos(id)).len();
            if len == 0 {
                println!("{} has no path", id);
            }
        }
    }

    pub fn name_of(&self, idx: usize) -> &str {
        &self.files[idx].name
    }

    fn calculate_path_of(&self, pos: FilePos) -> String {
        let mut result = String::new();
        let mut parents: Vec<FilePos> = Vec::new();
        let mut current = &self.files[pos.0];
        while !current.is_root() {
            let parent_pos = self.directories.get(&current.parent).expect(&format!("parent for {} not found", current.id.0));
            let parent = self.files.get(parent_pos.0).unwrap();
            parents.push(parent_pos.clone());
            current = parent;
        }
        for p in parents.into_iter().rev() {
            result.push_str(self.files.get(p.0).map(|k| &k.name).unwrap());
            result.push_str("\\");
        }
        result
    }

    pub fn search_by_name<'a, T>(&self, name: &'a str, items: T) -> Vec<usize>
        where T: IntoIterator<Item=usize> {
        let now = Instant::now();
        let mut result = Vec::new();
        for idx in items {
            let mut file_name = self.name_of(idx);
            if twoway::find_str(file_name, name).is_some() {
                result.push(idx);
            }
        }
        println!("total time {:?}", Instant::now().duration_since(now).subsec_nanos() / 1_000_000);
        result
    }
}

