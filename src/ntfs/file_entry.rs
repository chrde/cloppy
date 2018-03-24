use ntfs::attributes;
use ntfs::attributes::AttributeType;
use ntfs::FileRecordHeader;

const DOS_NAMESPACE: u8 = 2;

#[derive(Default, Debug)]
pub struct FileEntry {
    pub id: u32,

    pub name: String,
    pub fr_number: u64,
    pub dos_flags: u32,
    pub dos_flags1: u32,
    pub parent_id: u64,
    real_size: u64,
    logical_size: u64,
    modified_date: u64,
    created_date: u64,
    pub dataruns: Vec<attributes::Datarun>,
}

impl FileEntry {
    pub fn new(attrs: Vec<attributes::Attribute>, id: u32, seq_number: u16) -> Self {
        let mut result = FileEntry::default();
        result.id = id;
        result.fr_number = id as u64 | (seq_number as u64) << 48;
        //TODO handle attribute flags (e.g: sparse or compressed)
        attrs.into_iter().fold(result, |mut acc, attr| {
            match attr.attr_type {
                AttributeType::Standard(val) => {
                    acc.dos_flags1 = val.dos_flags;
                    acc.modified_date = val.modified;
                    acc.created_date = val.created;
                    acc
                }
                AttributeType::Filename(val) => {
                    if val.namespace != DOS_NAMESPACE {
                        acc.name = val.name;
                        acc.parent_id = val.parent_id;
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

