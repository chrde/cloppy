use sql::FileEntity;

pub struct ArenaFile {
    id: u32,
    name: usize,
    name_length: usize,
    path: i64,
    size: i64,
}

impl ArenaFile {
    pub fn path(&self) -> i64 {
        self.path
    }

    pub fn size(&self) -> i64 {
        self.size
    }
}

#[derive(Default)]
pub struct Arena {
    data: Vec<u8>,
    files: Vec<ArenaFile>,
}

unsafe impl Send for Arena {}

impl Arena {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_file(&mut self, f: FileEntity) {
        let (name, name_length) = self.add_data(f.name.into_bytes());
        let file = ArenaFile {
            id: f.id,
            name,
            name_length,
            path: f.path,
            size: f.size,
        };
        self.files.push(file);
    }
    fn add_data(&mut self, src: Vec<u8>) -> (usize, usize) {
        let field = self.data.len();
        self.data.extend(src);
        (field, self.data.len() - field)
    }

    pub fn file(&self, pos: usize) -> Option<&ArenaFile> {
       self.files.get(pos)
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn sort_by_name(&mut self) {
        let data = &self.data;
        self.files.sort_unstable_by_key(|k| (&data[k.name..k.name + k.name_length], k.id));
    }

    pub fn name_of(&self, pos: usize) -> &str {
        self.files.get(pos).map(|k| ::std::str::from_utf8(&self.data[k.name..k.name + k.name_length]).unwrap()).unwrap()
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

