use ntfs::attributes;
use ntfs::attributes::AttributeType;
use ntfs::file_record::FileRecordHeader;

const DOS_NAMESPACE: u8 = 2;

#[derive(Default, Debug, PartialEq)]
pub struct FileEntry {
    pub id: u32,
    pub flags: u16,
    pub fr_number: i64,
    pub base_record: i64,
    pub real_size: i64,
    pub modified_date: i64,
    pub created_date: i64,
    pub dataruns: Vec<attributes::Datarun>,
    pub names: Vec<FileEntryName>,
}

impl FileEntry {
    pub fn is_unused(&self) -> bool {
        self.flags % 2 == 0 || self.names.is_empty()
    }

    pub fn is_directory(&self) -> bool {
        self.flags == 3
    }

    pub fn has_name(&self) -> bool {
        !(self.names.len() == 1 && self.names[0].namespace == DOS_NAMESPACE)
    }

    pub fn is_candidate_for_fixes(&self) -> bool {
        self.base_record != 0 && self.is_directory()
    }

    pub fn requires_name_fix(&self) -> bool {
        self.is_directory() && !self.has_name() && self.base_record == 0
    }

    pub fn new(attrs: Vec<attributes::Attribute>, header: FileRecordHeader) -> Self {
        let mut result = FileEntry::default();
        result.id = header.fr_number;
        result.base_record = header.base_record as i64;
        result.flags = header.flags;
        result.fr_number = header.fr_number as i64 | (header.seq_number as i64) << 48;
        //TODO handle attribute flags (e.g: sparse or compressed)
        let entry = attrs.into_iter().fold(result, |mut acc, attr| {
            match attr.attr_type {
                AttributeType::Standard(val) => {
                    acc.modified_date = val.modified;
                    acc.created_date = val.created;
                }
                AttributeType::Filename(val) => {
                    let name = FileEntryName { name: val.name, namespace: val.namespace, parent_id: val.parent_id as u32, dos_flags: val.dos_flags };
                    acc.names.push(name);
                }
                AttributeType::Data(val) => {
                    acc.dataruns = val.datarun;
                    acc.real_size = val.size;
                }
            }
            acc
        });
        entry
    }
}


#[derive(Debug, PartialEq)]
pub struct FileEntryName {
    pub name: String,
    pub namespace: u8,
    pub parent_id: u32,
    pub dos_flags: u32,
}

