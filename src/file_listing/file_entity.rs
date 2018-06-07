use rusqlite::Result;
use rusqlite::Row;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct FileEntity {
    name: String,
    parent_id: FileId,
    size: i64,
    id: FileId,
    _id: usize,
    flags: u8,
}


#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct FileId(usize);

impl FileEntity {
    pub fn from_file_row(row: &Row) -> Result<Self> {
        let _id = row.get::<i32, u32>(0) as usize;
        let id = FileId(row.get::<i32, u32>(1) as usize);
        let parent_id = FileId(row.get::<i32, i64>(2) as usize);
        let size = row.get::<i32, i64>(4);
        let name = row.get::<i32, String>(5);
        let flags = row.get::<i32, u8>(8);
        Ok(FileEntity { name, parent_id, size, id, _id, flags })
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

    pub fn flags(&self) -> u8 {
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

