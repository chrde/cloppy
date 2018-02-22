pub use self::data::*;
pub use self::filename::*;
pub use self::standard::*;

mod data;
mod filename;
mod standard;

const ATTR_HEADER_SIZE: u32 = 0x10;
//attr_length and attr_offset
const RESIDENT_HEADER_SIZE: u32 = ATTR_HEADER_SIZE + 4 + 2;
//starting_vcn, last_vcn and datarun_offset
const NONRESIDENT_HEADER_SIZE: u32 = ATTR_HEADER_SIZE + 8 + 8 + 2;

#[derive(Debug, PartialEq)]
pub enum AttributeType {
    Standard(StandardAttr),
    Filename(FilenameAttr),
    Data(Vec<Datarun>),
    Ignored,
}

#[derive(Debug)]
pub struct Attribute {
    pub attr_flags: u16,
    pub attr_type: AttributeType,
}

