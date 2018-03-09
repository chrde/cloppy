use nom::{IResult, le_u16, le_u32};
use ntfs::FileRecordHeader;
use ntfs::attributes::Attribute;
use ntfs::attributes::AttributeType;
const END: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
use ntfs::attributes::standard::standard_attr;
use ntfs::attributes::filename::filename_attr;

fn file_record_header(input: &[u8]) -> IResult<&[u8], FileRecordHeader> {
    do_parse!(input,
        tag!(b"FILE") >>
        take!(2) >>
        fixup_size: le_u16 >>
        take!(12) >>
        attr_offset: le_u16 >>
        take!(22) >>
        fr_number: le_u32 >>
        fixup_seq: take!(2 * fixup_size) >>
        (FileRecordHeader{
            fr_number,
            attr_offset: attr_offset as usize,
            fixup_seq: fixup_seq.to_vec(),
        })
    )
}

fn attributes<T>(input: &[u8], attr_parser: T) -> IResult<&[u8], Attribute>
    where T: Fn(&[u8], u32) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        current_pos: curr_position >>
        attr_type: le_u32 >>
        attr_length: le_u32 >>
        take!(4) >>
        flags: le_u16 >>
        take!(2) >>
        attr: call!(attr_parser, attr_type) >>
        new_pos: curr_position >>
        take!(attr_length - (current_pos - new_pos)) >>
        (Attribute{
            attr_flags: flags,
            attr_type: attr,
        })
    )
}

fn parse_attributes<T>(attributes_parser: T, input: &[u8]) -> IResult<&[u8], (Vec<Attribute>, &[u8])>
    where T: Fn(&[u8], u32) -> IResult<&[u8], AttributeType> {
    many_till!(input, call!(attributes, &attributes_parser), tag!(END))
}

fn curr_position(input: &[u8]) -> IResult<&[u8], u32> {
    IResult::Done(input, input.len() as u32)
}
fn without_dataruns(input: &[u8], attr_type: u32) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        attribute_type: switch!(value!(attr_type),
                    0x10 => call!(standard_attr) |
                    0x30 => call!(filename_attr) |
                    _ => value!(AttributeType::Ignored)) >>
        (attribute_type)
    )
}

fn with_dataruns(input: &[u8], attr_type: u32) -> IResult<&[u8], AttributeType> {
    do_parse!(input,
        attribute_type: switch!(value!(attr_type),
                    0x10 => call!(standard_attr) |
                    0x30 => call!(filename_attr) |
                    0x80 => call!(data_attr) |
                    _ => value!(AttributeType::Ignored)) >>
        (attribute_type)
    )
}

