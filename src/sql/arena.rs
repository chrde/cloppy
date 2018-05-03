use sql::FileEntity;

pub struct ArenaFile {
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
            name,
            path,
            size,
        };
        self.files.push(file);
        let position = self.files.len() - 1;
    }
    fn add_data(&mut self, src: Vec<u8>) -> usize {
        let field = self.data.len();
        self.data.extend(src);
        field
    }

    pub fn name_of(&self, file: usize) -> &str {
        let f = &self.files[file];
        ::std::str::from_utf8(&self.data[f.name..f.path]).unwrap()
    }

    pub fn sort_by_name(&mut self) {
        let data = &self.data;
        self.files.sort_unstable_by_key(|k| &data[k.name..k.path]);
    }

    pub fn print(&self) {
//        for f in self.files.iter().take(10){
//            let name = ::std::str::from_utf8(&self.data[f.name..f.size]).unwrap();
//            println!("{}", name);
//        }
        for x in 0..10 {
            println!("{}", self.name_of(x + 130000));
        }
    }
}


