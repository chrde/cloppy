use sql::FileEntity;

#[derive(Default, Clone, Debug)]
pub struct ArenaFile {
    id: FileId,
    name: String,
    parent: FileId,
    size: i64,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialOrd, PartialEq, Hash)]
struct FileId(usize);

#[derive(Copy, Clone, Debug)]
struct FilePos(usize);

impl ArenaFile {
    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn is_root(&self) -> bool {
        self.parent == self.id
    }

    pub fn is_in_use(&self) -> bool {
        self.parent.0 != 0
    }
}

pub struct Arena {
    files: Vec<ArenaFile>,
    sorted_view: Vec<FilePos>,
}

unsafe impl Send for Arena {}

impl Arena {
    pub fn new(count: usize) -> Self {
        let sorted_view = Vec::with_capacity(count);
        let files = Vec::with_capacity(count);
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
            self.sorted_view.push(FilePos(self.files.len()));
            self.files.push(file);
        }
    }

    fn get_file(&self, pos: FilePos) -> Option<&ArenaFile> {
        self.files.get(pos.0)
    }

    pub fn file(&self, idx: usize) -> Option<&ArenaFile> {
        let pos = self.sorted_view[idx];
        self.get_file(pos)
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn sort_by_name(&mut self) {
        let files = &self.files;
        let sorted_view = &mut self.sorted_view;
        sorted_view.sort_unstable_by(|x, y| {
            let f1 = files.get(x.0).unwrap();
            let f2 = files.get(y.0).unwrap();
            f1.name.cmp(&f2.name)
        })
//        self.files.sort_unstable_by(|x, y| x.name.cmp(&y.name));
    }

    pub fn path_of(&self, idx: usize) -> String {
        let pos = self.sorted_view[idx];
        "".to_string()
//        self.calculate_path_of(pos)
    }

    pub fn name_of(&self, idx: usize) -> &str {
        let pos = self.sorted_view[idx];
       self.get_file(pos).map(|k| &k.name).unwrap()
    }

//    fn calculate_path_of(&self, id: FileId) -> String {
//        let mut result = String::new();
//        let mut parents: Vec<FileId> = Vec::new();
//        let mut current = self.files.get(id).unwrap();
//        while !current.is_root() {
//            let parent = self.files.get(current.parent).unwrap();
//            parents.push(parent.id.clone());
//            current = parent;
//        }
//        for p in parents.into_iter().rev() {
//            result.push_str(self.files.get(p).map(|k| &k.name).unwrap());
//            result.push_str("\\");
//        }
//        result
//    }

    pub fn search_by_name<'a, T>(&self, name: &'a str, items: T) -> Vec<usize>
        where T: IntoIterator<Item=usize> {
        let mut results = Vec::new();
        for idx in items {
            if self.name_of(idx).contains(name) {
                results.push(idx);
            }
        }
        results
    }
}

