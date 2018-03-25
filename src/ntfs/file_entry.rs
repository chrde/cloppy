use ntfs::attributes;
use ntfs::attributes::AttributeType;
use ntfs::FileRecordHeader;

const DOS_NAMESPACE: u8 = 2;

#[derive(Default, Debug)]
pub struct FileEntry {
    pub id: u32,

    pub name: String,
    pub fr_number: i64,
    pub parent_fr: i64,
    pub dos_flags: u32,
    pub real_size: i64,
    pub logical_size: i64,
    pub modified_date: i64,
    pub created_date: i64,
    pub dataruns: Vec<attributes::Datarun>,
}

impl FileEntry {
    pub fn new(attrs: Vec<attributes::Attribute>, id: u32, seq_number: u16) -> Self {
        let mut result = FileEntry::default();
        result.id = id;
        result.fr_number = id as i64 | (seq_number as i64) << 48;
        //TODO handle attribute flags (e.g: sparse or compressed)
        attrs.into_iter().fold(result, |mut acc, attr| {
            match attr.attr_type {
                AttributeType::Standard(val) => {
                    acc.modified_date = val.modified;
                    acc.created_date = val.created;
                    acc
                }
                AttributeType::Filename(val) => {
                    if val.namespace != DOS_NAMESPACE {
                        acc.name = val.name;
                        acc.parent_fr = val.parent_id;
                        acc.real_size = val.real_size;
                        acc.logical_size = val.allocated_size;
                        acc.dos_flags = val.flags;
                    }
                    acc
                }
                AttributeType::Data(val) => {
                    acc.dataruns = val;
                    acc
                }
            }
        })
    }
}

