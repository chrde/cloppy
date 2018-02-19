use super::{AttributeType, RESIDENT_HEADER_SIZE};
use nom::{le_u16, le_u32, le_u64, IResult};

#[derive(Debug)]
pub struct StandardAttr {
    pub dos_flags: u32,
    pub modified: u64,
    pub created: u64,
}

pub fn standard_attr(input: &[u8]) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        take!(4) >>
        attr_offset: le_u16 >>
        skipped_values: value!(attr_offset as u32 - RESIDENT_HEADER_SIZE) >>
        take!(skipped_values) >>
        created: le_u64 >>
        modified: le_u64 >>
        take!(16) >>
        dos_flags: le_u32 >>
        (AttributeType::Standard(
                StandardAttr{
                    modified: modified,
                    created: created,
                    dos_flags: dos_flags
                }))
    )
}

