use sql::FileEntity;
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct ArenaFile {
    id: FileId,
    name: String,
    parent: FileId,
    size: i64,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialOrd, PartialEq, Hash)]
//type FileId = usize;
struct FileId(usize);

impl ArenaFile {
    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn is_root(&self) -> bool {
        self.parent.0 == self.id.0
    }

    pub fn is_in_use(&self) -> bool {
        self.parent.0 != 0
    }
}

pub struct Arena {
    files: HashMap<FileId, ArenaFile>,
    sorted_view: Vec<FileId>,
}

unsafe impl Send for Arena {}

impl Arena {
    pub fn new(count: usize) -> Self {
        let sorted_view = Vec::with_capacity(count);
        let files = HashMap::with_capacity(count);
        Arena { files, sorted_view }
    }
    pub fn add_file(&mut self, f: FileEntity) {
        let file = ArenaFile {
            id: FileId(f.id),
            name: f.name,
            parent: FileId(f.parent_id),
            size: f.size,
        };
        if file.is_in_use() {
            self.sorted_view.push(FileId(f.id));
            self.files.insert(FileId(f.id), file);
        }
    }

    pub fn file(&self, pos: usize) -> Option<&ArenaFile> {
        let id = &self.sorted_view[pos];
        self.files.get(id)
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn sort_by_name(&mut self) {
//        self.files.sort_unstable_by(|x, y| x.name.cmp(&y.name));
    }

    pub fn path_of(&self, pos: usize) -> String {
        let id = &self.sorted_view[pos];
        "".to_string()
//        self.calculate_path_of(id)
    }

    pub fn name_of(&self, pos: usize) -> &str {
        let id = &self.sorted_view[pos];
        self.files.get(id).map(|k| &k.name).unwrap()
    }

    fn calculate_path_of(&self, id: &FileId) -> String {
        let mut result = String::new();
        let mut parents: Vec<FileId> = Vec::new();
        let mut current = self.files.get(id).unwrap();
        while !current.is_root() {
            let parent = self.files.get(&current.parent).unwrap();
            parents.push(parent.id.clone());
            current = parent;
        }
        for p in parents.into_iter().rev() {
            result.push_str(self.files.get(&p).map(|k| &k.name).unwrap());
            result.push_str("\\");
        }
        result
    }

    pub fn search_by_name<'a, T>(&self, name: &'a str, items: T) -> Vec<usize>
        where T: IntoIterator<Item=usize> {
        let mut results = Vec::new();
        for f in items {
            if self.name_of(f).contains(name) {
                results.push(f);
            }
        }
        results
    }
}

