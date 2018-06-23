use ntfs::file_entry::FileEntry;
use rusqlite::Result;
use rusqlite::Row;
use std::u32;

const DOS_NAMESPACE: u8 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct FileEntity {
    name: String,
    parent_id: FileId,
    size: i64,
    id: FileId,
    _id: u32,
    flags: u16,
}


#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct FileId {
    id: u32,
}

impl FileId {
    pub fn new(id: u32) -> FileId {
        FileId { id }
    }
}

impl FileEntity {
    pub fn from_file_row(row: &Row) -> Result<FileEntity> {
        let _id = row.get::<i32, u32>(0);
        let id = FileId::new(row.get::<i32, u32>(1));
        let parent_id = FileId::new(row.get::<i32, i64>(2) as u32);
        let size = row.get::<i32, i64>(4);
        let name = row.get::<i32, String>(5);
        let flags = row.get::<i32, u16>(8);
        Ok(FileEntity { name, parent_id, size, id, _id, flags })
    }

    pub fn from_file_entry(file: FileEntry) -> FileEntity {
        let name = file.names.into_iter()
            .filter(|n| n.namespace != DOS_NAMESPACE)
            .take(1)
            .next()
            .expect(&format!("Found a file record without name: {}", file.fr_number));
        FileEntity {
            name: name.name,
            parent_id: FileId::new(name.parent_id),
            size: file.real_size,
            id: FileId::new(file.id),
            _id: u32::MAX,
            flags: file.flags,
        }
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

}

