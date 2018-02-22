use nom::{IResult, le_u16, le_u32, le_u64};
use super::{AttributeType, RESIDENT_HEADER_SIZE};

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_standard_attr() {
        let a = [
            72, 0, 0, 0,
            24, 0,
            0, 0,
            82, 131, 14, 254, 172, 15, 209, 1,
            82, 131, 14, 254, 172, 15, 209, 1,
            82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1,
            6, 0, 0, 0,
            99, 99];

        let remainder = [99, 99];
        let result = AttributeType::Standard(StandardAttr {
            dos_flags: 6,
            modified: 130903099841610578,
            created: 130903099841610578,
        });

        assert_eq!(standard_attr(&a[..]), IResult::Done(&remainder[..], result));// Ok(AttributeType::Ignored, &[99, 99])));
    }
}