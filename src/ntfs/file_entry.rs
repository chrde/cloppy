use ntfs::attributes;
use ntfs::attributes::AttributeType;
use ntfs::file_record::FileRecordHeader;

const DOS_NAMESPACE: u8 = 2;

#[derive(Default, Debug, PartialEq)]
pub struct FileEntry {
    pub id: u32,
    pub flags: u16,
    pub name: String,
    pub fr_number: i64,
    pub parent_id: u32,
    pub dos_flags: u32,
    pub real_size: i64,
    pub modified_date: i64,
    pub created_date: i64,
    pub dataruns: Vec<attributes::Datarun>,
}

impl FileEntry {
    pub fn is_in_use(&self) -> bool {
        return self.flags != 0 && self.id != 0;
    }

    pub fn new(attrs: Vec<attributes::Attribute>, header: FileRecordHeader) -> Self {
        let mut result = FileEntry::default();
        result.id = header.fr_number;
        result.flags = header.flags;
        result.fr_number = header.fr_number as i64 | (header.seq_number as i64) << 48;
        //TODO handle attribute flags (e.g: sparse or compressed)
        let entry = attrs.into_iter().fold(result, |mut acc, attr| {
            match attr.attr_type {
                AttributeType::Standard(val) => {
                    acc.modified_date = val.modified;
                    acc.created_date = val.created;
                    acc
                }
                AttributeType::Filename(val) => {
                    if val.namespace != DOS_NAMESPACE {
                        acc.name = val.name;
                        acc.parent_id = val.parent_id as u32;
                        acc.dos_flags = val.flags;
                    }
                    acc
                }
                AttributeType::Data(val) => {
                    acc.dataruns = val.datarun;
                    acc.real_size = val.size;
                    acc
                }
            }
        });
        entry
    }
}

