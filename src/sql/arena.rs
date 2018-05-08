use sql::FileEntity;

pub struct ArenaFile {
    id: u32,
    name: usize,
    path: usize,
    size: usize,
}

#[derive(Default)]
pub struct Arena {
    data: Vec<u8>,
    pub files: Vec<ArenaFile>,
}

unsafe impl Send for Arena {}

impl Arena {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_file(&mut self, f: FileEntity) {
        let name = self.add_data(f.name.into_bytes());
        let path = self.add_data(f.path);
        let size = self.add_data(f.size);
        let file = ArenaFile {
            id: f.id,
            name,
            path,
            size,
        };
        self.files.push(file);
    }
    fn add_data(&mut self, src: Vec<u8>) -> usize {
        let field = self.data.len();
        self.data.extend(src);
        field
    }

    pub fn sort_by_name(&mut self) {
        let data = &self.data;
        self.files.sort_unstable_by_key(|k| (&data[k.name..k.path], k.id));
    }

    pub fn name_of(&self, pos: usize) -> &str {
        self.files.get(pos).map(|f| ::std::str::from_utf8(&self.data[f.name..f.path]).unwrap()).unwrap()
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

