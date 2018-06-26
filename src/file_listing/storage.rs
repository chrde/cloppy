use file_listing::file_entity::FileEntity;
use file_listing::file_entity::FileId;
use file_listing::file_entity::FileType;
use file_listing::files::FileData;
use file_listing::files::NameId;
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::iter::Chain;
use std::iter::Iterator;
use std::mem;
use std::slice::Iter;

pub struct Storage {
    file_data: Vec<FileData>,
    dir_data: Vec<FileData>,
    names: Vec<String>,
}

impl Storage {
    pub fn new() -> Storage {
        let file_data = Vec::new();
        let dir_data = Vec::new();
        let names = Vec::new();
        Storage {
            file_data,
            dir_data,
            names,
        }
    }

    pub fn bulk_insert(&mut self, files: Vec<FileEntity>) {
        let names = files.iter().map(|f| f.name().to_string()).collect::<BTreeSet<String>>();
        {
            let mut names_idx: HashMap<&str, u32> = HashMap::new();

            for (pos, name) in names.iter().enumerate() {
                names_idx.insert(name, pos as u32);
            }
            let mut files = files.into_iter()
                .map(|f| {
                    let name_id = names_idx.get(f.name()).unwrap();
                    let mut data: FileData = f.into();
                    data.set_name_id(NameId(*name_id));
                    data
                })
                .collect::<Vec<FileData>>();
            files.sort_unstable_by_key(|f| f.id());
            for f in files {
                if f.is_directory() {
                    self.dir_data.push(f);
                } else {
                    self.file_data.push(f);
                }
            }
        }

        mem::replace(&mut self.names, names.into_iter().collect());
    }

    pub fn upsert<T: Into<String>>(&mut self, mut data: FileData, name: T) {
        let files = if data.is_directory() {
            &mut self.dir_data
        } else {
            &mut self.file_data
        };
        match files.binary_search_by_key(&data.id(), |f| f.id()) {
            Ok(pos) => {
                let name = name.into();
                println!("UPDATE {:?} {:?}", data, name);
                let old_data = files.get_mut(pos).unwrap();
                let old_name = self.names.get_mut(old_data.name_id().0 as usize).unwrap();
                mem::replace(old_name, name);
                data.set_name_id(old_data.name_id());
                mem::replace(old_data, data);
            }
            Err(pos) => {
                let name_pos = self.names.len();
                self.names.push(name.into());
                data.set_name_id(NameId(name_pos as u32));
                files.insert(pos, data)
            }
        };
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn get<T: Borrow<FileId>>(&self, id: T) -> StorageItem {
        let files = match id.borrow().f_type() {
            FileType::DIRECTORY => &self.dir_data,
            FileType::FILE => &self.file_data,
        };
        let pos = files.binary_search_by_key(id.borrow(), |f| f.id()).unwrap();
        let data = files.get(pos).unwrap();
        let name = self.names.get(data.name_id().0 as usize).unwrap();
        StorageItem {
            data,
            name,
        }
    }

    pub fn iter(&self) -> StorageIter {
        let dir_iter = self.dir_data.iter();
        let file_iter = self.file_data.iter();
        let inner = dir_iter.chain(file_iter);
        StorageIter {
            names: &self.names,
            inner,
        }
    }
}

pub struct StorageItem<'a> {
    pub name: &'a str,
    pub data: &'a FileData,
}

pub struct StorageIter<'a> {
    names: &'a [String],
    inner: Chain<Iter<'a, FileData>, Iter<'a, FileData>>,
}

impl<'a> Iterator for StorageIter<'a> {
    type Item = StorageItem<'a>;

    fn next(&mut self) -> Option<StorageItem<'a>> {
        self.inner.next().map(|data| {
            StorageItem {
                name: self.names.get(data.name_id().0 as usize).unwrap(),
                data,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use file_listing::file_entity::FileId;
    use super::*;

    const FILE: u16 = 1;
    const DIR: u16 = 2;

    fn test_data() -> Storage {
        let mut storage = Storage::new();
        let file0 = FileData::new(FileId::file(0), FileId::directory(1), 0, FILE, false);
        let dir0 = FileData::new(FileId::directory(0), FileId::directory(1), 0, DIR, false);
        let file1 = FileData::new(FileId::file(1), FileId::directory(1), 0, FILE, false);
        let dir1 = FileData::new(FileId::directory(1), FileId::directory(1), 0, DIR, false);
        let file2 = FileData::new(FileId::file(2), FileId::directory(1), 0, FILE, false);
        let dir2 = FileData::new(FileId::directory(2), FileId::directory(1), 0, DIR, false);
        let dir3 = FileData::new(FileId::directory(3), FileId::directory(1), 0, DIR, false);

        storage.upsert(file2, "file2");
        storage.upsert(file1, "file1");
        storage.upsert(file0, "file0");
        storage.upsert(dir3, "dir3");
        storage.upsert(dir2, "dir2");
        storage.upsert(dir1, "dir1");
        storage.upsert(dir0, "dir0");

        storage
    }

    #[test]
    fn adding_files_keeps_order_by_file_id() {
        let storage = test_data();

        for x in 0..3 {
            let StorageItem { name, data } = storage.get(FileId::file(x));
            assert_eq!(name, format!("file{}", x));
            assert_eq!(FileId::file(x), data.id());
        }
    }

    #[test]
    fn can_update_file_and_name() {
        let mut storage = test_data();

        storage.upsert(FileData::new(FileId::file(2), FileId::directory(1), 25, FILE, false), "new_file2");

        let StorageItem { name, data } = storage.get(FileId::file(2));
        assert_eq!(name, "new_file2");
        assert_eq!(data.size(), 25);
    }

    #[test]
    fn adding_dir_keeps_order_by_file_id() {
        let storage = test_data();

        for x in 0..3 {
            let StorageItem { name, data } = storage.get(FileId::directory(x));
            assert_eq!(name, format!("dir{}", x));
            assert_eq!(FileId::directory(x), data.id());
        }
    }

    #[test]
    fn can_update_dir_and_name() {
        let mut storage = test_data();

        let updated_dir = FileData::new(FileId::directory(2), FileId::directory(1), 25, DIR, false);
        storage.upsert(updated_dir, "new_dir2");

        let StorageItem { name, data } = storage.get(FileId::directory(2));
        assert_eq!(name, "new_dir2");
        assert_eq!(data.size(), 25);
    }

    #[test]
    fn iterator_over_dirs_and_files() {
        let storage = test_data();

        let dirs = storage.iter().filter(|i| i.data.is_directory()).count();
        assert_eq!(4, dirs);

        let files = storage.iter().filter(|i| !i.data.is_directory()).count();
        assert_eq!(3, files);
    }

    #[test]
    fn iterates_first_over_dirs() {
        let storage = test_data();

        let dirs = storage.iter().take(4).filter(|i| i.data.is_directory()).count();
        assert_eq!(4, dirs);
    }
}