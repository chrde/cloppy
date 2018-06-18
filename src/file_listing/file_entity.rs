use ntfs::file_entry::FileEntry;
use rusqlite::Result;
use rusqlite::Row;
use std::cmp::Ordering;
use std::usize;

const DOS_NAMESPACE: u8 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct FileEntity {
    name: String,
    parent_id: FileId,
    deleted: bool,
    size: i64,
    id: FileId,
    _id: usize,
    flags: u16,
}


#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct FileId {
    id: usize,
}

impl FileId {
    pub fn new(id: usize) -> FileId {
        FileId { id }
    }
}

impl FileEntity {
    pub fn from_file_row(row: &Row) -> Result<FileEntity> {
        let _id = row.get::<i32, u32>(0) as usize;
        let id = FileId::new(row.get::<i32, u32>(1) as usize);
        let parent_id = FileId::new(row.get::<i32, i64>(2) as usize);
        let size = row.get::<i32, i64>(4);
        let name = row.get::<i32, String>(5);
        let flags = row.get::<i32, u16>(8);
        let deleted = false;
        Ok(FileEntity { name, deleted, parent_id, size, id, _id, flags })
    }

    pub fn from_file_entry(file: FileEntry) -> FileEntity {
        let name = file.names.into_iter()
            .filter(|n| n.namespace != DOS_NAMESPACE)
            .take(1)
            .next()
            .expect(&format!("Found a file record without name: {}", file.fr_number));
        FileEntity {
            name: name.name,
            parent_id: FileId::new(name.parent_id as usize),
            size: file.real_size,
            deleted: false,
            id: FileId::new(file.id as usize),
            _id: usize::MAX,
            flags: file.flags,
        }
    }

    pub fn set_deleted(&mut self, deleted: bool) {
        self.deleted = deleted;
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn parent_id(&self) -> FileId {
        self.parent_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }

    pub fn is_root(&self) -> bool {
        self.parent_id == self.id
    }

    pub fn is_directory(&self) -> bool {
        self.flags & 0x02 != 0
    }

    pub fn name_comparator(x: &FileEntity, y: &FileEntity) -> Ordering {
        x.name.cmp(&y.name)
    }
}

