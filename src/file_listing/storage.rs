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

    fn update_file_name_ids(&mut self, new_name_id: NameId) {
        self.file_data.iter_mut()
            .chain(self.dir_data.iter_mut())
            .filter(|f| f.name_id().0 >= new_name_id.0)
            .for_each(|f| {
                let old = f.name_id().0;
                f.set_name_id(NameId(old + 1));
            })
    }

    fn upsert_name<T: Into<String>>(&mut self, name: T) -> NameId {
        let name = name.into();
        match self.names.binary_search(&name) {
            Ok(pos) => NameId(pos as u32),
            Err(pos) => {
                self.names.insert(pos, name);
                let new = NameId(pos as u32);
                self.update_file_name_ids(new);
                new
            }
        }
    }

    pub fn upsert<T: Into<String>>(&mut self, mut data: FileData, name: T) {
        let new_name_id = self.upsert_name(name);
        data.set_name_id(new_name_id);
        let files = match data.is_directory() {
            true => &mut self.dir_data,
            false => &mut self.file_data,
        };
        match files.binary_search_by_key(&data.id(), |f| f.id()) {
            Ok(pos) => {
                let old_data = files.get_mut(pos).unwrap();
                mem::replace(old_data, data);
            }
            Err(pos) => {
                files.insert(pos, data)
            }
        };
    }

    pub fn delete<T: Borrow<FileId>>(&mut self, id: T) {
        let files = match id.borrow().f_type() {
            FileType::DIRECTORY => &mut self.dir_data,
            FileType::FILE => &mut self.file_data,
        };
        match files.binary_search_by_key(id.borrow(), |f| f.id()) {
            Err(_) => println!("Delete file\tNot found\t{:?}", id.borrow()),
            Ok(pos) => files.get_mut(pos).unwrap().set_deleted(true),
        }
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
        let dir3 = FileData::new(FileId::directory(3), FileId::directory(2), 0, DIR, false);

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

    #[test]
    fn mark_file_as_deleted() {
        let mut storage = test_data();

        assert!(!storage.get(FileId::file(1)).data.deleted());
        storage.delete(FileId::file(1));

        assert!(storage.get(FileId::file(1)).data.deleted());
    }

    #[test]
    fn adding_file_with_existing_name_does_not_add_new_name() {
        let mut storage = test_data();
        let prev_name_len = storage.names.len();
        let prev_file_len = storage.file_data.len();

        let file = FileData::new(FileId::file(5), FileId::directory(1), 0, FILE, false);
        storage.upsert(file, "file2");

        assert_eq!(prev_name_len, storage.names.len());
        assert_eq!(prev_file_len + 1, storage.file_data.len());
    }

    #[test]
    fn update_file_data_same_name() {
        let mut storage = test_data();
        let prev_name_len = storage.names.len();

        let file = FileData::new(FileId::file(1), FileId::directory(1), 25, FILE, true);
        storage.upsert(file, "file1");

        assert_eq!(prev_name_len, storage.names.len());
        assert_eq!(25, storage.get(FileId::file(1)).data.size());
        assert!(storage.get(FileId::file(1)).data.deleted());
    }


    #[test]
    fn update_file_data_new_existing_name() {
        let mut storage = test_data();
        let prev_name_len = storage.names.len();

        let file = FileData::new(FileId::file(1), FileId::directory(1), 25, FILE, true);
        storage.upsert(file, "file1");

        assert_eq!(prev_name_len, storage.names.len());
        assert_eq!(25, storage.get(FileId::file(1)).data.size());
        assert!(storage.get(FileId::file(1)).data.deleted());
    }

    #[test]
    fn update_file_data_new_name() {
        let mut storage = test_data();
        let prev_name_len = storage.names.len();

        let update_file = FileData::new(FileId::file(1), FileId::directory(1), 25, FILE, true);
        storage.upsert(update_file, "file_update");

        assert_eq!(prev_name_len + 1, storage.names.len());
        assert_eq!(25, storage.get(FileId::file(1)).data.size());
        assert!(storage.get(FileId::file(1)).data.deleted());
    }

    #[test]
    fn update_file_does_not_change_existing_files() {
        let mut storage = test_data();
        let new_file = FileData::new(FileId::file(4), FileId::directory(1), 42, FILE, false);
        storage.upsert(new_file, "aaa_file");
        let prev_name_len = storage.names.len();

        let update_file = FileData::new(FileId::file(1), FileId::directory(1), 25, FILE, true);
        storage.upsert(update_file, "file_update");

        assert_eq!(prev_name_len + 1, storage.names.len());
        assert_eq!("aaa_file", storage.get(FileId::file(4)).name);
        assert_eq!(42, storage.get(FileId::file(4)).data.size());
    }

    #[test]
    #[ignore]
    fn old_names_are_removed() {}
}